import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.AbstractCard {
    id: root
    property string auditId: ""
    property string timestamp: ""
    property string actionType: ""
    property string summary: ""
    property string entryStatus: "unknown"  // Success, Failed, Pending
    property string actor: ""
    property string prompt: ""
    property string rationale: ""
    property var filesChanged: []
    property string rollbackHint: ""
    property string resultMessage: ""

    signal rollbackClicked()

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        // Header row: status icon + summary
        RowLayout {
            Layout.fillWidth: true

            Kirigami.Icon {
                source: {
                    if (entryStatus === "Success") return "dialog-ok"
                    if (entryStatus === "Failed") return "dialog-cancel"
                    return "emblem-important"
                }
                implicitWidth: Kirigami.Units.iconSizes.small
                implicitHeight: Kirigami.Units.iconSizes.small
                color: {
                    if (entryStatus === "Success") return "#4caf50"
                    if (entryStatus === "Failed") return "#f44336"
                    return "#ff9800"
                }
            }

            Label {
                Layout.fillWidth: true
                text: summary
                font.weight: Font.Medium
                elide: Text.ElideRight
                maximumLineCount: 1
            }

            Label {
                text: formatTimestamp(timestamp)
                color: Kirigami.Theme.disabledTextColor
                font.pointSize: 9
            }
        }

        // Expandable details section
        Rectangle {
            id: detailsSection
            Layout.fillWidth: true
            visible: expandBtn.checked
            color: Kirigami.Theme.backgroundColor
            radius: Kirigami.Units.smallSpacing
            border.color: Kirigami.Theme.separatorColor
            border.width: 1
            implicitHeight: detailsColumn.implicitHeight + Kirigami.Units.smallSpacing * 2

            ColumnLayout {
                id: detailsColumn
                anchors.fill: parent
                anchors.margins: Kirigami.Units.smallSpacing
                spacing: Kirigami.Units.smallSpacing

                Label {
                    visible: prompt.length > 0
                    text: "Prompt: " + prompt
                    wrapMode: Text.Wrap
                    font.family: "monospace"
                    font.pointSize: 9
                    color: Kirigami.Theme.complementaryTextColor
                }

                Label {
                    visible: rationale.length > 0
                    text: "Rationale: " + rationale
                    wrapMode: Text.Wrap
                    font.pointSize: 9
                    color: Kirigami.Theme.complementaryTextColor
                }

                Label {
                    visible: filesChanged.length > 0
                    text: "Files: " + filesChanged.join(", ")
                    wrapMode: Text.Wrap
                    font.pointSize: 9
                    color: Kirigami.Theme.complementaryTextColor
                }

                Label {
                    visible: resultMessage.length > 0
                    text: "Result: " + resultMessage
                    wrapMode: Text.Wrap
                    font.pointSize: 9
                }

                Label {
                    visible: rollbackHint.length > 0
                    text: "Rollback: " + rollbackHint
                    wrapMode: Text.Wrap
                    font.pointSize: 9
                    color: Kirigami.Theme.neutralTextColor
                }

                Button {
                    visible: actionType === "Apply" || actionType === "Rollback"
                    text: "Rollback"
                    flat: true
                    Kirigami.Theme.textColor: Kirigami.Theme.neutralTextColor
                    onClicked: root.rollbackClicked()
                }
            }
        }

        // Expand toggle
        Button {
            id: expandBtn
            text: checked ? "Show less" : "Show details"
            checkable: true
            flat: true
            Layout.fillWidth: true
            visible: prompt.length > 0 || rationale.length > 0 || filesChanged.length > 0
        }
    }

    function formatTimestamp(ts) {
        if (!ts || ts.length === 0) return ""
        // Show relative time or shortened format
        // Expected format: ISO 8601
        try {
            var date = new Date(ts)
            var now = new Date()
            var diffMs = now - date
            var diffMin = Math.floor(diffMs / 60000)
            if (diffMin < 1) return "just now"
            if (diffMin < 60) return diffMin + "m ago"
            var diffHr = Math.floor(diffMin / 60)
            if (diffHr < 24) return diffHr + "h ago"
            return date.toLocaleDateString()
        } catch(e) {
            return ts
        }
    }
}
