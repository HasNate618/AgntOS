import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.AbstractCard {
    id: root
    property string proposalId: ""
    property string summary: ""

    property var onApprove: null
    property var onReject: null

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true

            Kirigami.Icon {
                source: "dialog-warning"
                implicitWidth: Kirigami.Units.iconSizes.medium
                implicitHeight: Kirigami.Units.iconSizes.medium
                color: Kirigami.Theme.neutralTextColor
            }

            Label {
                Layout.fillWidth: true
                text: "Apply " + proposalId + "?"
                font.weight: Font.Bold
                color: Kirigami.Theme.neutralTextColor
            }
        }

        Label {
            Layout.fillWidth: true
            text: summary
            wrapMode: Text.Wrap
            font.pointSize: 10
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: Kirigami.Units.smallSpacing

            Button {
                text: "✓ Approve"
                Kirigami.Theme.textColor: "#ffffff"
                background: Rectangle {
                    color: "#4caf50"
                    radius: Kirigami.Units.smallSpacing
                }
                Layout.fillWidth: true
                onClicked: {
                    if (root.onApprove) root.onApprove(proposalId)
                }
            }

            Button {
                text: "✗ Reject"
                Kirigami.Theme.textColor: "#ffffff"
                background: Rectangle {
                    color: "#f44336"
                    radius: Kirigami.Units.smallSpacing
                }
                Layout.fillWidth: true
                onClicked: {
                    if (root.onReject) root.onReject(proposalId)
                }
            }
        }
    }
}
