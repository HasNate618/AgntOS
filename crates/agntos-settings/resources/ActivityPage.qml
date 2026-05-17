import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Page {
    id: root
    title: "Activity"

    property string lastQuery: ""

    Component.onCompleted: appBridge.load_audit(50)

    onVisibleChanged: {
        if (visible && lastQuery.length === 0) {
            appBridge.load_audit(50)
        }
    }

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
                Layout.preferredHeight: 36
                placeholderText: "Search audit log…"
                onAccepted: {
                    lastQuery = text
                    appBridge.search_audit(text)
                }
                onTextChanged: {
                    if (text.length === 0 && lastQuery.length > 0) {
                        lastQuery = ""
                        appBridge.load_audit(50)
                    }
                }
            }

            Button {
                text: "↻ Refresh"
                implicitHeight: 36
                onClicked: {
                    searchField.text = ""
                    lastQuery = ""
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
                spacing: 6

                delegate: Pane {
                    width: auditList.width
                    padding: 10

                    ColumnLayout {
                        spacing: 4

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 8

                            Rectangle {
                                width: 10; height: 10; radius: 5
                                color: {
                                    var s = (status || "").toString().toLowerCase()
                                    if (s === "success") return "#4caf50"
                                    if (s === "error" || s === "failed") return "#f44336"
                                    if (s === "pending") return "#ff9800"
                                    return "#9e9e9e"
                                }
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
                                text: {
                                    var id = auditId || ""
                                    return id.length > 8 ? id.substring(0, 8) + "…" : id
                                }
                                font.family: "monospace"
                                font.pointSize: 8
                                color: "#888888"
                            }

                            Item { Layout.fillWidth: true }

                            Label {
                                text: actor || ""
                                font.pointSize: 9
                                color: "#aaaaaa"
                                visible: (actor || "").length > 0
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            visible: expandBtn.checked && prompt !== undefined && (prompt || "").length > 0
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
                            visible: prompt !== undefined && (prompt || "").length > 0
                        }
                    }
                }
            }
        }
    }
}
