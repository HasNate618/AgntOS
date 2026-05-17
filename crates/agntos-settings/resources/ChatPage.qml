import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Page {
    id: root

    SystemPalette { id: palette; colorGroup: SystemPalette.Active }

    ColumnLayout {
        anchors.fill: parent
        spacing: 4

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
                        anchors { left: String(entryType) === "user" ? undefined : parent.left; right: String(entryType) === "user" ? parent.right : undefined }
                        anchors.leftMargin: String(entryType) === "user" ? 48 : 8
                        anchors.rightMargin: String(entryType) === "user" ? 8 : 48
                        width: Math.min(contentColumn.implicitWidth + 16, parent.width * 0.75)
                        height: contentColumn.implicitHeight + 16
                        radius: 8
                        color: {
                            if (String(entryType) === "user") return palette.highlight
                            if (String(entryType) === "approval") return "#fff3cd"
                            if (String(entryType) === "tool_call" || String(entryType) === "tool_result") return palette.alternateBase
                            return palette.base
                        }
                        border.width: String(entryType) === "assistant" ? 1 : 0
                        border.color: palette.mid

                        ColumnLayout {
                            id: contentColumn
                            anchors.fill: parent
                            anchors.margins: 8
                            spacing: 4

                            Text {
                                Layout.fillWidth: true
                                text: {
                                    if (String(entryType) === "tool_call") return "🔧 " + (toolName || "") + "..."
                                    if (String(entryType) === "tool_result") return "✓ " + (toolName || "") + " completed"
                                    if (String(entryType) === "approval") return "⚠️ Apply " + (proposalId || "") + "?"
                                    return content || ""
                                }
                                wrapMode: Text.Wrap
                                color: String(entryType) === "user" ? palette.highlightedText : palette.text
                            }

                            Text {
                                visible: String(entryType) === "approval"
                                text: proposalSummary || ""
                                wrapMode: Text.Wrap
                                color: "#856404"
                                Layout.fillWidth: true
                            }

                            RowLayout {
                                visible: String(entryType) === "approval"
                                Layout.fillWidth: true
                                spacing: 8

                                Button {
                                    text: "Approve"
                                    background: Rectangle { color: "#4caf50"; radius: 4 }
                                    contentItem: Text { text: "✓ Approve"; color: "#ffffff"; verticalAlignment: Text.AlignVCenter; horizontalAlignment: Text.AlignHCenter }
                                    onClicked: appBridge.approve_proposal(proposalId)
                                }
                                Button {
                                    text: "Reject"
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

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 8
            spacing: 8

            TextField {
                id: inputField
                Layout.fillWidth: true
                placeholderText: appBridge.is_processing ? "Waiting for agent..." : "Ask the agent..."
                enabled: true
                onAccepted: sendChat()
            }

            Button {
                text: "Send"
                enabled: inputField.text.trim().length > 0 && !appBridge.is_processing
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
