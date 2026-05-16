import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Page {
    id: root
    title: "Activity"

    property string searchQuery: ""

    Connections {
        target: appBridge
        onAuditChanged: {}
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: 8
            spacing: 8

            TextField {
                id: searchField
                Layout.fillWidth: true
                placeholderText: "Search audit log..."
                onAccepted: {
                    searchQuery = text
                    appBridge.search_audit(text)
                }
                onTextChanged: {
                    if (text.length === 0 && searchQuery.length > 0) {
                        searchQuery = ""
                        appBridge.load_audit(50)
                    }
                }
            }

            Button {
                text: "↻ Refresh"
                onClicked: {
                    searchField.text = ""
                    searchQuery = ""
                    appBridge.load_audit(50)
                }
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: auditList
                model: appBridge.audit_items
                spacing: 8

                delegate: Pane {
                    width: auditList.width
                    padding: 12

                    ColumnLayout {
                        spacing: 4

                        RowLayout {
                            Layout.fillWidth: true

                            Rectangle {
                                width: 10; height: 10; radius: 5
                                color: entryStatus === "Success" ? "#4caf50" : (entryStatus === "Failed" ? "#f44336" : "#ff9800")
                            }

                            Label {
                                Layout.fillWidth: true
                                text: summary || ""
                                font.weight: Font.Medium
                                elide: Text.ElideRight
                                maximumLineCount: 1
                            }

                            Label {
                                text: timestamp || ""
                                color: "#888888"
                                font.pointSize: 9
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8

                            Label {
                                text: actionType || ""
                                font.pointSize: 9
                                color: "#555555"
                            }

                            Label {
                                text: auditId || ""
                                font.family: "monospace"
                                font.pointSize: 8
                                color: "#888888"
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            visible: expandBtn.checked && prompt !== undefined && prompt.length > 0
                            color: "#f5f5f5"
                            radius: 4
                            border.color: "#cccccc"
                            border.width: 1
                            height: promptLabel.implicitHeight + 8

                            Label {
                                id: promptLabel
                                anchors.fill: parent
                                anchors.margins: 4
                                text: "Prompt: " + (prompt || "")
                                wrapMode: Text.Wrap
                                font.family: "monospace"
                                font.pointSize: 9
                                color: "#555555"
                            }
                        }

                        Button {
                            id: expandBtn
                            text: checked ? "Show less" : "Show details"
                            checkable: true
                            flat: true
                            Layout.fillWidth: true
                            visible: prompt !== undefined && prompt.length > 0
                        }
                    }
                }
            }
        }
    }
}
