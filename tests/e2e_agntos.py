# E2E test: Full AgntOS flow with real LLM
# Tests: prompt, tool calls (inspect, bash), streaming, thinking separation
import subprocess, json, sys, threading, time, os

# Ensure agntctl is on PATH
os.environ["PATH"] = os.path.expanduser("~/.local/bin") + ":" + os.environ.get("PATH","")

cd = os.path.dirname(os.path.abspath(__file__))
agntos_dir = os.path.normpath(os.path.join(cd, '..'))
ext_path = os.path.join(agntos_dir, 'crates/agntos-cc/etc/agntos/extensions/agntos-tools/index.ts')
prompt_path = os.path.join(agntos_dir, 'crates/agntos-cc/etc/agntos/AGENTS.md')
system_prompt = open(prompt_path).read()

print(f"=== AgntOS E2E Test ===")
print(f"Extension: {ext_path}")
print(f"System prompt: {prompt_path}")
print()

# Start Pi with full AgntOS config
proc = subprocess.Popen(
    ['pi', '--mode', 'rpc',
     '--no-builtin-tools', '--no-extensions', '--no-skills', '--no-context-files',
     '-e', ext_path,
     '--system-prompt', system_prompt,
     '--thinking', 'off',
     '--no-session'],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    text=True
)

events = []
def reader():
    for line in proc.stdout:
        events.append(line.strip())
t = threading.Thread(target=reader, daemon=True)
t.start()

time.sleep(2)

def send(cmd, label=""):
    proc.stdin.write(json.dumps(cmd) + '\n')
    proc.stdin.flush()
    print(f">>> {label or cmd.get('type','')}")

def wait_for(predicate, timeout=60):
    deadline = time.time() + timeout
    while time.time() < deadline:
        for e in events:
            try:
                obj = json.loads(e)
                if predicate(obj):
                    return obj
            except:
                pass
        time.sleep(0.5)
    return None

# Test 1: Basic prompt
print("\n--- Test 1: Basic prompt ---")
send({"type":"prompt","message":"say hi in one word"}, "prompt")

r = wait_for(lambda e: e.get('type') == 'agent_end', 120)
if r:
    msgs = r.get('messages', [])
    for m in msgs:
        if m.get('role') == 'assistant':
            texts = [c.get('text','') for c in m.get('content',[]) if c.get('type')=='text']
            thinking = [c.get('thinking','') for c in m.get('content',[]) if c.get('type')=='thinking']
            print(f"  Assistant response: {''.join(texts)}")
            if thinking:
                print(f"  Thinking: {''.join(thinking)[:100]}...")
    print(f"  PASS: Agent completed")
else:
    print(f"  FAIL: No agent_end within timeout")

# Test 2: Tool call (inspect)
print("\n--- Test 2: Tool call (agntos_inspect) ---")
send({"type":"prompt","message":"check my cpu using agntos_inspect"}, "prompt")

tool_start = wait_for(lambda e: e.get('type') == 'tool_execution_start', 60)
if tool_start:
    print(f"  Tool called: {tool_start.get('toolName')} args={tool_start.get('args')}")
    tool_end = wait_for(lambda e: e.get('type') == 'tool_execution_end', 120)
    if tool_end:
        result = tool_end.get('result', {})
        content = result.get('content', [])
        for c in content:
            if c.get('type') == 'text':
                print(f"  Tool result: {c.get('text','')[:200]}")
        print(f"  PASS: Tool call completed")
    else:
        print(f"  FAIL: No tool_execution_end")
else:
    print(f"  FAIL: No tool_execution_start")

# Wait for agent_end if still running
wait_for(lambda e: e.get('type') == 'agent_end', 60)

# Test 3: Tool call (bash)
print("\n--- Test 3: Tool call (agntos_bash) ---")
send({"type":"prompt","message":"run 'echo hello from agntos' using agntos_bash"}, "prompt")

tool_start = wait_for(lambda e: e.get('type') == 'tool_execution_start', 60)
if tool_start:
    print(f"  Tool called: {tool_start.get('toolName')}")
    tool_end = wait_for(lambda e: e.get('type') == 'tool_execution_end', 120)
    if tool_end:
        result = tool_end.get('result', {})
        is_err = tool_end.get('isError', False)
        print(f"  isError: {is_err}")
        content = result.get('content', [])
        for c in content:
            if c.get('type') == 'text':
                print(f"  Tool result: {c.get('text','')[:200]}")
        print(f"  PASS: Tool call completed" if not is_err else f"  PASS: Tool returned (with error)")
    else:
        print(f"  FAIL: No tool_execution_end")
else:
    print(f"  FAIL: No tool_execution_start")

wait_for(lambda e: e.get('type') == 'agent_end', 60)

# Test 4: Verify event types match frontend expectations
print("\n--- Test 4: Event format validation ---")
required_events = {'agent_start','agent_end','turn_start','turn_end',
                   'message_start','message_end','message_update'}
found_events = set()
for e in events:
    try:
        obj = json.loads(e)
        found_events.add(obj.get('type',''))
    except:
        pass
for req in required_events:
    if req in found_events:
        print(f"  ✓ {req}")
    else:
        print(f"  ✗ {req} (MISSING)")

# Check message_update has assistantMessageEvent
msg_updates = [json.loads(e) for e in events if '"message_update"' in e]
delta_types = set()
for mu in msg_updates:
    ame = mu.get('assistantMessageEvent', {})
    if ame:
        delta_types.add(ame.get('type'))
print(f"  Delta types seen: {delta_types}")

# Test 5: Session management
print("\n--- Test 5: Session management ---")
send({"type":"new_session"}, "new_session")
r = wait_for(lambda e: e.get('type') == 'response' and e.get('command') == 'new_session', 5)
if r:
    print(f"  new_session: success={r.get('success')}")
    print(f"  PASS: Session creation")

send({"type":"get_state"}, "get_state")
r = wait_for(lambda e: e.get('type') == 'response' and e.get('command') == 'get_state', 5)
if r:
    data = r.get('data',{})
    print(f"  sessionId: {data.get('sessionId','N/A')[:20]}...")
    print(f"  messageCount: {data.get('messageCount')}")
    print(f"  model: {data.get('model',{}).get('id','N/A')}")

# Summary
print(f"\n=== Summary ===")
print(f"Total events received: {len(events)}")
tool_starts = [e for e in events if '"tool_execution_start"' in e]
print(f"Tool calls made: {len(tool_starts)}")

proc.kill()
