import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.ApplicationWindow {
    id: root
    title: "AgntOS Control Center"
    width: 900
    height: 700
    minimumWidth: 400
    minimumHeight: 300

    property bool connected: appBridge ? appBridge.connected : false

    Connections {
        target: appBridge
        function onStatusChanged() { root.connected = appBridge.connected }
    }

    globalDrawer: Kirigami.GlobalDrawer {
        title: "AgntOS"
        titleIcon: "system-run"

        actions: [
            Kirigami.Action {
                text: "Chat"
                icon.name: "chat-bubbles"
                onTriggered: pageStack.layers.clear()
                checked: pageStack.currentIndex === 0
            },
            Kirigami.Action {
                text: "Status"
                icon.name: "computer"
                onTriggered: pageStack.layers.clear()
                checked: pageStack.currentIndex === 1
            },
            Kirigami.Action {
                text: "Proposals"
                icon.name: "document-edit"
                onTriggered: pageStack.layers.clear()
                checked: pageStack.currentIndex === 2
            },
            Kirigami.Action {
                text: "Activity"
                icon.name: "view-history"
                onTriggered: pageStack.layers.clear()
                checked: pageStack.currentIndex === 3
            }
        ]

        DrawerFooter {
            RowLayout {
                anchors.centerIn: parent
                spacing: Kirigami.Units.smallSpacing

                StatusIndicator {
                    connected: root.connected
                }
                Label {
                    text: root.connected ? "Connected" : "Disconnected"
                    color: root.connected ? "#4caf50" : "#f44336"
                    font.pointSize: 10
                }
            }
        }
    }

    pageStack.initialPage: ChatPage {
        id: chatPage
        chatModel: appBridge.chatItems
        isProcessing: appBridge.isProcessing

        function sendChat(text) {
            appBridge.send_chat(text)
        }
    }

    pageStack.extendedLayers: [
        StatusPage {
            id: statusPage
            statusModel: appBridge
            onRefreshRequested: appBridge.refresh_status()
        },
        ProposalsPage {
            id: proposalsPage
            proposalModel: appBridge.proposalItems
            onApply: function(id) { appBridge.approve_proposal(id) }
            onDismiss: function(id) { appBridge.dismiss_proposal(id) }
            onRefreshRequested: appBridge.refresh_proposals()
        },
        ActivityPage {
            id: activityPage
            auditModel: appBridge.auditItems
            onSearch: function(query) { appBridge.search_audit(query) }
            onRefreshRequested: appBridge.load_audit(50)
        }
    ]
}
