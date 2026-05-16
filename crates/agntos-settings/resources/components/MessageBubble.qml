import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Rectangle {
    id: root
    property string text: ""
    property bool isUser: false

    implicitWidth: Math.min(textLabel.implicitWidth + Kirigami.Units.largeSpacing * 2, parent.width * 0.75)
    implicitHeight: textLabel.implicitHeight + Kirigami.Units.largeSpacing * 2

    radius: Kirigami.Units.smallSpacing * 2
    color: isUser ? Kirigami.Theme.highlightColor : Kirigami.Theme.backgroundColor
    border.color: isUser ? "transparent" : Kirigami.Theme.separatorColor
    border.width: isUser ? 0 : 1

    Label {
        id: textLabel
        anchors.fill: parent
        anchors.margins: Kirigami.Units.smallSpacing
        text: root.text
        wrapMode: Text.Wrap
        color: isUser ? Kirigami.Theme.highlightedTextColor : Kirigami.Theme.textColor
        textFormat: Text.PlainText
    }
}
