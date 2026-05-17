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
                enabled: appBridge.connected
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
                    id: card
                    width: proposalList.width
                    padding: 12

                    property bool expanded: false

                    TapHandler {
                        onTapped: card.expanded = !card.expanded
                    }

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
                                text: {
                                    var s = summary || "";
                                    if (s === "") {
                                        var changes = nixChanges || "";
                                        return changes.length > 80 ? changes.substring(0, 80) + "…" : changes || "Untitled";
                                    }
                                    return s;
                                }
                                font.weight: Font.Bold
                                wrapMode: Text.Wrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                            }

                            Label {
                                text: status || "pending"
                                color: status === "applied" ? "#4caf50" : (status === "pending" ? "#f5a623" : "#888888")
                                font.pointSize: 9
                                font.weight: Font.Bold
                            }
                        }

                        Label {
                            Layout.fillWidth: true
                            text: {
                                if (summary && summary !== "") return "";
                                var changes = nixChanges || "";
                                return changes;
                            }
                            wrapMode: Text.Wrap
                            maximumLineCount: expanded ? 100 : 2
                            elide: Text.ElideRight
                            visible: summary === "" || summary === undefined || expanded
                            color: "#666666"
                            font.pointSize: 10
                        }

                        Label {
                            Layout.fillWidth: true
                            text: createdAt || ""
                            color: "#999999"
                            font.pointSize: 9
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.topMargin: 4
                            height: expanded ? detailsColumn.implicitHeight + 16 : 0
                            clip: true
                            color: "#f5f5f5"
                            radius: 4
                            visible: expanded
                            Behavior on height { NumberAnimation { duration: 200 } }

                            ColumnLayout {
                                id: detailsColumn
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.top: parent.top
                                anchors.margins: 8
                                spacing: 6

                                Label {
                                    text: "Nix Changes"
                                    font.weight: Font.Bold
                                    font.pointSize: 10
                                }

                                Label {
                                    Layout.fillWidth: true
                                    text: nixChanges || "(none)"
                                    font.family: "monospace"
                                    font.pointSize: 9
                                    wrapMode: Text.Wrap
                                    color: "#444444"
                                }

                                Label {
                                    text: "Rollback"
                                    font.weight: Font.Bold
                                    font.pointSize: 10
                                    visible: rollbackGuidance && rollbackGuidance !== ""
                                }

                                Label {
                                    Layout.fillWidth: true
                                    text: rollbackGuidance || ""
                                    font.family: "monospace"
                                    font.pointSize: 9
                                    wrapMode: Text.Wrap
                                    color: "#444444"
                                    visible: rollbackGuidance && rollbackGuidance !== ""
                                }
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Layout.topMargin: 4
                            spacing: 8
                            visible: expanded

                            Button {
                                text: "Apply"
                                enabled: status === "pending"
                                background: Rectangle {
                                    color: enabled ? "#4caf50" : "#cccccc"
                                    radius: 4
                                }
                                contentItem: Text {
                                    text: "Apply"
                                    color: enabled ? "#ffffff" : "#888888"
                                    verticalAlignment: Text.AlignVCenter
                                    horizontalAlignment: Text.AlignHCenter
                                }
                                onClicked: appBridge.approve_proposal(proposalId)
                            }

                            Button {
                                text: "Dismiss"
                                enabled: status === "pending"
                                background: Rectangle {
                                    color: enabled ? "#888888" : "#cccccc"
                                    radius: 4
                                }
                                contentItem: Text {
                                    text: "Dismiss"
                                    color: enabled ? "#ffffff" : "#888888"
                                    verticalAlignment: Text.AlignVCenter
                                    horizontalAlignment: Text.AlignHCenter
                                }
                                onClicked: appBridge.dismiss_proposal(proposalId)
                            }
                        }
                    }
                }
            }
        }
    }
}
