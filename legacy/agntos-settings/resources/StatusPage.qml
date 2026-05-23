import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Page {
    id: root
    title: "Status"

    Connections {
        target: appBridge
        onStatusChanged: {}
    }

    function connectionColor(state) {
        if (state === "connected") return "#4caf50"
        if (state === "connecting") return "#ff9800"
        return "#f44336"
    }

    function connectionIcon(state) {
        if (state === "connected") return "●"
        if (state === "connecting") return "◐"
        return "○"
    }

    ScrollView {
        anchors.fill: parent
        clip: true

        ColumnLayout {
            spacing: 12
            anchors.margins: 12
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top

            RowLayout {
                Layout.fillWidth: true

                Label {
                    text: "System Status"
                    font.pointSize: 16
                    font.weight: Font.Bold
                }

                Item { Layout.fillWidth: true }

                Button {
                    text: "↻ Refresh"
                    onClicked: appBridge.refresh_status()
                }
            }

            Pane {
                Layout.fillWidth: true
                padding: 12

                ColumnLayout {
                    spacing: 4

                    Label { text: "Agent"; font.weight: Font.Bold; font.pointSize: 14 }

                    GridLayout {
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 4
                        Layout.fillWidth: true

                        Label { text: "Connection:"; font.weight: Font.Bold }
                        Label {
                            text: connectionIcon(appBridge.connection_state) + " " + appBridge.connection_state
                            color: connectionColor(appBridge.connection_state)
                            font.capitalization: Font.Capitalize
                        }
                        Label { text: "Profile:"; font.weight: Font.Bold }
                        Label { text: appBridge.profile_name || "—" }
                        Label { text: "Model:"; font.weight: Font.Bold }
                        Label { text: appBridge.model_name || "—" }
                    }
                }
            }

            Pane {
                Layout.fillWidth: true
                padding: 12

                ColumnLayout {
                    spacing: 4

                    Label { text: "System"; font.weight: Font.Bold; font.pointSize: 14 }

                    GridLayout {
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 4
                        Layout.fillWidth: true

                        Label { text: "CPU:"; font.weight: Font.Bold }
                        Label { text: appBridge.cpu_info || "—" }
                        Label { text: "RAM:"; font.weight: Font.Bold }
                        Label { text: appBridge.ram_used || "—" }
                        Label { text: "Disk:"; font.weight: Font.Bold }
                        Label { text: appBridge.disk_used || "—" }
                        Label { text: "Failed units:"; font.weight: Font.Bold }
                        Label {
                            text: Number(appBridge.failed_units).toString() || "0"
                            color: Number(appBridge.failed_units) > 0 ? "#f44336" : "#4caf50"
                        }
                    }
                }
            }

            Pane {
                Layout.fillWidth: true
                padding: 12

                ColumnLayout {
                    spacing: 4

                    Label { text: "Watchdog"; font.weight: Font.Bold; font.pointSize: 14 }

                    GridLayout {
                        columns: 2
                        columnSpacing: 12
                        rowSpacing: 4
                        Layout.fillWidth: true

                        Label { text: "Alert count:"; font.weight: Font.Bold }
                        Label {
                            text: Number(appBridge.watchdog_alert_count).toString() || "0"
                            color: Number(appBridge.watchdog_alert_count) > 0 ? "#ff9800" : "#4caf50"
                        }
                        Label { text: "Last check:"; font.weight: Font.Bold }
                        Label { text: appBridge.last_check_time || "—" }
                    }
                }
            }
        }
    }
}
