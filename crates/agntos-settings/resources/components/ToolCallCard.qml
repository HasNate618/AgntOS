import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.AbstractCard {
    id: root
    property string toolName: ""
    property string toolStatus: "running"
    property string toolOutput: ""
    property bool success: true

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true

            Kirigami.Icon {
                source: {
                    if (toolStatus === "running") return "emblem-downloads"
                    if (success) return "dialog-ok"
                    return "dialog-cancel"
                }
                implicitWidth: Kirigami.Units.iconSizes.small
                implicitHeight: Kirigami.Units.iconSizes.small
                color: {
                    if (toolStatus === "running") return Kirigami.Theme.neutralTextColor
                    if (success) return "#4caf50"
                    return "#f44336"
                }
            }

            BusyIndicator {
                id: spinner
                running: toolStatus === "running"
                visible: toolStatus === "running"
                implicitWidth: Kirigami.Units.iconSizes.small
                implicitHeight: Kirigami.Units.iconSizes.small
            }

            Label {
                Layout.fillWidth: true
                text: {
                    if (toolStatus === "running") return capitalize(toolName) + "..."
                    if (success) return capitalize(toolName) + " completed"
                    return capitalize(toolName) + " failed"
                }
                elide: Text.ElideRight
                font.weight: Font.Medium
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: outputArea.implicitHeight + Kirigami.Units.smallSpacing * 2
            visible: toolStatus === "done" && toolOutput.length > 0
            color: Kirigami.Theme.backgroundColor
            radius: Kirigami.Units.smallSpacing
            border.color: Kirigami.Theme.separatorColor
            border.width: 1
            clip: true

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.smallSpacing

                Label {
                    id: outputArea
                    Layout.fillWidth: true
                    text: toolOutput
                    wrapMode: Text.Wrap
                    maximumLineCount: visible ? 5 : 0
                    elide: Text.ElideRight
                    font.family: "monospace"
                    font.pointSize: 9
                    color: Kirigami.Theme.disabledTextColor
                }

                Button {
                    id: showMoreBtn
                    text: "Show more"
                    visible: toolOutput.length > 300
                    flat: true
                    onClicked: outputArea.maximumLineCount = 0
                }
            }
        }
    }

    function capitalize(str) {
        if (str.length === 0) return str
        return str.charAt(0).toUpperCase() + str.slice(1)
    }
}
