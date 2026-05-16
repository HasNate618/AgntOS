import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.ScrollablePage {
    id: root
    title: "Chat"

    property var chatModel: null
    property bool isProcessing: false

    header: Kirigami.InlineMessage {
        id: errorMessage
        visible: false
        type: Kirigami.MessageType.Error
        showCloseButton: true
    }

    actions: [
        Kirigami.Action {
            icon.name: "edit-clear"
            tooltip: "Clear chat"
            onTriggered: chatModel.clear()
            enabled: chatModel && chatModel.count > 0
        }
    ]

    ListView {
        id: chatList
        model: chatModel
        clip: true
        spacing: Kirigami.Units.largeSpacing
        verticalLayoutDirection: ListView.BottomToTop

        onCountChanged: {
            if (count > 0) {
                positionViewAtBeginning()
            }
        }

        delegate: Item {
            width: chatList.width
            height: delegateLoader.height + Kirigami.Units.largeSpacing

            Loader {
                id: delegateLoader
                width: parent.width
                sourceComponent: {
                    if (model.entryType === "user") return userComponent
                    if (model.entryType === "assistant") return assistantComponent
                    if (model.entryType === "tool_call" || model.entryType === "tool_result") return toolComponent
                    if (model.entryType === "approval") return approvalComponent
                    return unknownComponent
                }
            }
        }
    }

    footer: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: Kirigami.Units.smallSpacing

            TextField {
                id: inputField
                Layout.fillWidth: true
                placeholderText: isProcessing ? "Waiting for agent..." : "Ask the agent..."
                enabled: !isProcessing
                Keys.onReturnPressed: sendMessage()
                selectByMouse: true
            }

            Button {
                icon.name: "send-email"
                enabled: inputField.text.trim().length > 0 && !isProcessing
                onClicked: sendMessage()
            }
        }
    }

    function sendMessage() {
        var text = inputField.text.trim()
        if (text.length === 0) return
        inputField.text = ""
        if (root.sendChat) {
            root.sendChat(text)
        }
    }

    Component {
        id: userComponent
        MessageBubble {
            text: model.content
            isUser: true
            anchors.right: parent.right
            anchors.rightMargin: Kirigami.Units.largeSpacing
        }
    }

    Component {
        id: assistantComponent
        MessageBubble {
            text: model.content
            isUser: false
            anchors.left: parent.left
            anchors.leftMargin: Kirigami.Units.largeSpacing
            opacity: 0
            Behavior on opacity { NumberAnimation { duration: 80; easing.type: Easing.OutQuad } }
            Component.onCompleted: opacity = 1
        }
    }

    Component {
        id: toolComponent
        ToolCallCard {
            toolName: model.toolName || ""
            toolStatus: model.entryType === "tool_call" ? "running" : "done"
            toolOutput: model.content || ""
            success: model.toolSuccess || false
            anchors.left: parent.left
            anchors.leftMargin: Kirigami.Units.largeSpacing + Kirigami.Units.gridUnit
        }
    }

    Component {
        id: approvalComponent
        ApprovalCard {
            proposalId: model.proposalId || ""
            summary: model.proposalSummary || ""
            anchors.horizontalCenter: parent.horizontalCenter
        }
    }

    Component {
        id: unknownComponent
        MessageBubble {
            text: model.content || "(unknown)"
            isUser: false
            anchors.left: parent.left
        }
    }
}
