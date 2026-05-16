import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.ScrollablePage {
    id: root
    title: "Proposals"

    property var proposalModel: null
    property var onApply: null
    property var onDismiss: null
    signal refreshRequested()

    actions: [
        Kirigami.Action {
            icon.name: "view-refresh"
            tooltip: "Refresh proposals"
            onTriggered: root.refreshRequested()
        }
    ]

    Kirigami.CardsListView {
        model: proposalModel
        delegate: Kirigami.AbstractCard {
            contentItem: ColumnLayout {
                spacing: Kirigami.Units.smallSpacing

                RowLayout {
                    Layout.fillWidth: true

                    Kirigami.Icon {
                        source: model.status === "pending" ? "document-edit" : "dialog-ok"
                        implicitWidth: Kirigami.Units.iconSizes.small
                        implicitHeight: Kirigami.Units.iconSizes.small
                        color: model.status === "pending" ? Kirigami.Theme.neutralTextColor : "#4caf50"
                    }

                    Label {
                        Layout.fillWidth: true
                        text: model.proposalId || "?"
                        font.weight: Font.Bold
                        font.family: "monospace"
                    }

                    Label {
                        text: model.status || "pending"
                        color: model.status === "applied" ? "#4caf50" : Kirigami.Theme.neutralTextColor
                        font.pointSize: 9
                    }
                }

                Label {
                    Layout.fillWidth: true
                    text: model.summary || ""
                    wrapMode: Text.Wrap
                    elide: Text.ElideRight
                    maximumLineCount: 2
                }

                RowLayout {
                    Layout.fillWidth: true
                    visible: model.status === "pending"

                    Button {
                        text: "Apply"
                        background: Rectangle {
                            color: "#4caf50"
                            radius: Kirigami.Units.smallSpacing
                        }
                        Kirigami.Theme.textColor: "#ffffff"
                        onClicked: {
                            if (root.onApply) root.onApply(model.proposalId)
                        }
                    }

                    Button {
                        text: "Dismiss"
                        background: Rectangle {
                            color: Kirigami.Theme.disabledTextColor
                            radius: Kirigami.Units.smallSpacing
                        }
                        Kirigami.Theme.textColor: "#ffffff"
                        onClicked: {
                            if (root.onDismiss) root.onDismiss(model.proposalId)
                        }
                    }
                }
            }
        }
    }
}
