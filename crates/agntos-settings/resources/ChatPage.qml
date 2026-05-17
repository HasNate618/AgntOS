import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Page {
    id: root

    SystemPalette { id: palette; colorGroup: SystemPalette.Active }

    ColumnLayout {
        anchors.fill: parent
        spacing: 4

        // Status indicator row
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: turnStateLabel.visible ? 28 : 0
            color: {
                if (!appBridge.connected) return "#f44336"
                var ts = String(appBridge.turn_state)
                if (ts.indexOf("thinking") >= 0) return "#ffe082"
                if (ts.indexOf("awaiting_approval") >= 0) return "#fff3cd"
                if (ts.indexOf("error") >= 0) return "#ffcdd2"
                return palette.alternateBase
            }
            visible: !appBridge.connected || String(appBridge.turn_state) !== "idle"

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 8
                spacing: 6

                Rectangle {
                    width: 10; height: 10; radius: 5
                    color: {
                        if (!appBridge.connected) return "#ffffff"
                        var ts = String(appBridge.turn_state)
                        if (ts.indexOf("thinking") >= 0) return "#f57c00"
                        if (ts.indexOf("tool_running") >= 0) return "#2196f3"
                        if (ts.indexOf("awaiting_approval") >= 0) return "#ff9800"
                        if (ts.indexOf("error") >= 0) return "#f44336"
                        if (ts.indexOf("streaming") >= 0) return "#4caf50"
                        return "#888888"
                    }
                    SequentialAnimation on color {
                        running: {
                            var ts = String(appBridge.turn_state)
                            return ts.indexOf("thinking") >= 0 || ts.indexOf("tool_running") >= 0 || ts.indexOf("awaiting_approval") >= 0
                        }
                        loops: Animation.Infinite
                        PropertyAnimation { to: "#ffffff"; duration: 600 }
                        PropertyAnimation { to: {
                            var ts = String(appBridge.turn_state)
                            if (ts.indexOf("thinking") >= 0) return "#f57c00"
                            if (ts.indexOf("awaiting_approval") >= 0) return "#ff9800"
                            return "#2196f3"
                        }; duration: 600 }
                    }
                }

                Label {
                    id: turnStateLabel
                    text: {
                        if (!appBridge.connected) return "Disconnected — reconnecting..."
                        var ts = String(appBridge.turn_state)
                        if (ts === "idle") return ""
                        if (ts.indexOf("thinking") >= 0) return "Agent: Thinking..."
                        if (ts.indexOf("tool_running") >= 0) {
                            var parts = ts.split(":")
                            return "Agent: Running " + (parts[1] || "tool") + "..."
                        }
                        if (ts.indexOf("awaiting_approval") >= 0) return "Agent: Awaiting approval"
                        if (ts.indexOf("streaming") >= 0) return "Agent: Responding..."
                        if (ts.indexOf("completed") >= 0) return "Agent: Done"
                        if (ts.indexOf("error") >= 0) {
                            var msg = ts.substring(ts.indexOf(":") + 1)
                            return "Error: " + msg
                        }
                        return ""
                    }
                    color: !appBridge.connected ? "#ffffff" : palette.text
                    font.pointSize: 10
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }

        // Chat messages
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: chatList
                model: appBridge.chat_model
                spacing: 8
                verticalLayoutDirection: ListView.TopToBottom

                onCountChanged: {
                    chatList.positionViewAtEnd()
                }

                delegate: Item {
                    width: chatList.width
                    height: chatBubble.height + 8

                    Rectangle {
                        id: chatBubble
                        anchors {
                            left: isUser ? undefined : parent.left
                            right: isUser ? parent.right : undefined
                        }
                        anchors.leftMargin: isUser ? 48 : 8
                        anchors.rightMargin: isUser ? 8 : 48
                        width: Math.min(contentColumn.implicitWidth + 16, parent.width * 0.8)
                        height: contentColumn.implicitHeight + 16
                        radius: 8
                        color: {
                            var et = String(entryType)
                            if (et === "usermessage") return palette.highlight
                            if (et === "approvalrequest") return "#fff3cd"
                            if (et === "toolcall" || et === "toolresult") return palette.alternateBase
                            return palette.base
                        }
                        border.width: et === "assistanttext" ? 1 : 0
                        border.color: palette.mid

                        readonly property bool isUser: String(entryType) === "usermessage"
                        readonly property string et: String(entryType)

                        ColumnLayout {
                            id: contentColumn
                            anchors.fill: parent
                            anchors.margins: 8
                            spacing: 4

                            // User or assistant text
                            Text {
                                visible: String(entryType) === "usermessage" || String(entryType) === "assistanttext"
                                Layout.fillWidth: true
                                text: content || ""
                                wrapMode: Text.Wrap
                                color: String(entryType) === "usermessage" ? palette.highlightedText : palette.text
                            }

                            // Tool call (running)
                            RowLayout {
                                visible: String(entryType) === "toolcall" && String(toolStatus) === "running"
                                Layout.fillWidth: true
                                spacing: 6

                                BusyIndicator {
                                    running: true
                                    width: 16; height: 16
                                }

                                Text {
                                    text: "🔧 " + (toolName || "tool") + "..."
                                    color: palette.text
                                    font.pointSize: 10
                                }
                            }

                            // Tool result (done)
                            ColumnLayout {
                                visible: String(entryType) === "toolresult" || (String(entryType) === "toolcall" && String(toolStatus) === "done")
                                Layout.fillWidth: true
                                spacing: 4

                                RowLayout {
                                    Layout.fillWidth: true

                                    Text {
                                        text: toolSuccess === "true" || toolSuccess ? "✓" : "✗"
                                        color: toolSuccess === "true" || toolSuccess ? "#4caf50" : "#f44336"
                                        font.pointSize: 12
                                    }

                                    Text {
                                        Layout.fillWidth: true
                                        text: (toolName || "Tool") + " completed"
                                        color: palette.text
                                        font.pointSize: 10
                                        font.weight: Font.Bold
                                    }
                                }

                                Text {
                                    Layout.fillWidth: true
                                    text: content || ""
                                    wrapMode: Text.Wrap
                                    color: palette.text
                                    font.pointSize: 9
                                    maximumLineCount: 3
                                    elide: Text.ElideRight
                                }
                            }

                            // Approval request
                            ColumnLayout {
                                visible: String(entryType) === "approvalrequest"
                                Layout.fillWidth: true
                                spacing: 4

                                Text {
                                    Layout.fillWidth: true
                                    text: "⚠️ Apply " + (proposalId || "proposal") + "?"
                                    font.weight: Font.Bold
                                    color: "#856404"
                                }

                                Text {
                                    visible: proposalSummary !== ""
                                    Layout.fillWidth: true
                                    text: proposalSummary || ""
                                    wrapMode: Text.Wrap
                                    color: "#856404"
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    Button {
                                        text: "✓ Approve"
                                        background: Rectangle { color: "#4caf50"; radius: 4 }
                                        contentItem: Text { text: "✓ Approve"; color: "#ffffff"; verticalAlignment: Text.AlignVCenter; horizontalAlignment: Text.AlignHCenter }
                                        onClicked: appBridge.approve_proposal(proposalId)
                                    }
                                    Button {
                                        text: "✗ Reject"
                                        background: Rectangle { color: "#f44336"; radius: 4 }
                                        contentItem: Text { text: "✗ Reject"; color: "#ffffff"; verticalAlignment: Text.AlignVCenter; horizontalAlignment: Text.AlignHCenter }
                                        onClicked: appBridge.dismiss_proposal(proposalId)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Input row
        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 8
            spacing: 8

            TextField {
                id: inputField
                Layout.fillWidth: true
                placeholderText: {
                    if (!appBridge.connected) return "Disconnected..."
                    return "Ask the agent..."
                }
                enabled: true
                onAccepted: sendChat()
            }

            Button {
                text: "Send"
                enabled: inputField.text.trim().length > 0 && String(appBridge.turn_state) === "idle"
                onClicked: sendChat()
            }
        }
    }

    function sendChat() {
        var text = inputField.text.trim()
        if (text.length === 0) return
        inputField.text = ""
        appBridge.send_chat(text)
    }
}
