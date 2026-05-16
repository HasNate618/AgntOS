import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Page {
    id: root
    title: "Proposals"

    Connections {
        target: appBridge
        onProposalsChanged: {}
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 8

            Label {
                text: "Proposals"
                font.pointSize: 16
                font.weight: Font.Bold
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "↻ Refresh"
                onClicked: appBridge.refresh_proposals()
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: proposalList
                model: appBridge.proposal_items
                spacing: 8
                delegate: Pane {
                    width: proposalList.width
                    padding: 12

                    ColumnLayout {
                        spacing: 4

                        RowLayout {
                            Layout.fillWidth: true

                            Rectangle {
                                width: 10; height: 10; radius: 5
                                color: status === "pending" ? "#f5a623" : (status === "applied" ? "#4caf50" : "#888888")
                            }

                            Label {
                                Layout.fillWidth: true
                                text: proposalId || "?"
                                font.weight: Font.Bold
                                font.family: "monospace"
                            }

                            Label {
                                text: status || "pending"
                                color: status === "applied" ? "#4caf50" : (status === "pending" ? "#f5a623" : "#888888")
                                font.pointSize: 9
                            }
                        }

                        Label {
                            Layout.fillWidth: true
                            text: summary || ""
                            wrapMode: Text.Wrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            visible: status === "pending"
                            spacing: 8

                            Button {
                                text: "Apply"
                                background: Rectangle { color: "#4caf50"; radius: 4 }
                                contentItem: Text { text: "Apply"; color: "#ffffff"; verticalAlignment: Text.AlignVCenter; horizontalAlignment: Text.AlignHCenter }
                                onClicked: appBridge.approve_proposal(proposalId)
                            }

                            Button {
                                text: "Dismiss"
                                background: Rectangle { color: "#888888"; radius: 4 }
                                contentItem: Text { text: "Dismiss"; color: "#ffffff"; verticalAlignment: Text.AlignVCenter; horizontalAlignment: Text.AlignHCenter }
                                onClicked: appBridge.dismiss_proposal(proposalId)
                            }
                        }
                    }
                }
            }
        }
    }
}
