import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ApplicationWindow {
    id: root
    title: "AgntOS Control Center"
    width: 900
    height: 700
    visible: true

    // Poll for background thread updates (chat responses)
    Timer {
        interval: 100
        running: true
        repeat: true
        onTriggered: appBridge.poll_updates()
    }

    property int currentPage: 0

    header: TabBar {
        id: tabBar
        currentIndex: root.currentPage
        onCurrentIndexChanged: root.currentPage = currentIndex

        TabButton { text: "Chat" }
        TabButton { text: "Status" }
        TabButton { text: "Proposals" }
        TabButton { text: "Activity" }
    }

    footer: ToolBar {
        visible: !appBridge.connected
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 8

            Rectangle {
                width: 10; height: 10; radius: 5
                color: "#f44336"
            }

            Label {
                text: "Disconnected"
                color: "#f44336"
                font.pointSize: 10
            }

            Item { Layout.fillWidth: true }
        }
    }

    StackLayout {
        anchors.fill: parent
        currentIndex: root.currentPage

        ChatPage { 
        id: chatPage
    }
        StatusPage { }
        ProposalsPage { }
        ActivityPage { }
    }
}
