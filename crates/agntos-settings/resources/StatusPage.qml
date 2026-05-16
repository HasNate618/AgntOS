import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.ScrollablePage {
    id: root
    title: "Status"

    property var statusModel: null
    signal refreshRequested()

    actions: [
        Kirigami.Action {
            icon.name: "view-refresh"
            tooltip: "Refresh status"
            onTriggered: root.refreshRequested()
        }
    ]

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing

        Kirigami.Card {
            Layout.fillWidth: true
            title: "Agent"

            contentItem: GridLayout {
                columns: 2
                columnSpacing: Kirigami.Units.largeSpacing
                rowSpacing: Kirigami.Units.smallSpacing

                Label { text: "Status:"; font.weight: Font.Bold }
                Label {
                    text: statusModel ? (statusModel.connected ? "● Connected" : "○ Disconnected") : "—"
                    color: statusModel && statusModel.connected ? "#4caf50" : "#f44336"
                }
                Label { text: "Profile:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.profileName : "—" }
                Label { text: "Model:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.modelName : "—" }
                Label { text: "Endpoint:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.endpoint : "—"; elide: Text.ElideRight }
            }
        }

        Kirigami.Card {
            Layout.fillWidth: true
            title: "System"

            contentItem: GridLayout {
                columns: 2
                columnSpacing: Kirigami.Units.largeSpacing
                rowSpacing: Kirigami.Units.smallSpacing

                Label { text: "CPU:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.cpuInfo : "—" }
                Label { text: "RAM:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.ramUsed : "—" }
                Label { text: "Disk:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.diskUsed : "—" }
                Label { text: "Failed units:"; font.weight: Font.Bold }
                Label {
                    text: statusModel ? String(statusModel.failedUnits) : "—"
                    color: statusModel && statusModel.failedUnits > 0 ? "#f44336" : "#4caf50"
                }
            }
        }

        Kirigami.Card {
            Layout.fillWidth: true
            title: "Watchdog"

            contentItem: GridLayout {
                columns: 2
                columnSpacing: Kirigami.Units.largeSpacing
                rowSpacing: Kirigami.Units.smallSpacing

                Label { text: "Interval:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.watchdogInterval + "s" : "—" }
                Label { text: "Disk threshold:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.watchdogDiskThreshold + "%" : "—" }
                Label { text: "Alerts:"; font.weight: Font.Bold }
                Label {
                    text: statusModel ? String(statusModel.watchdogAlertCount) : "—"
                    color: statusModel && statusModel.watchdogAlertCount > 0 ? "#ff9800" : "#4caf50"
                }
                Label { text: "Last check:"; font.weight: Font.Bold }
                Label { text: statusModel ? statusModel.lastCheckTime : "—" }
            }
        }
    }
}
