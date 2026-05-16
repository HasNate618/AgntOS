import QtQuick 2.15

Rectangle {
    property bool connected: false
    width: 12
    height: 12
    radius: 6
    color: connected ? "#4caf50" : "#f44336"

    Behavior on color {
        ColorAnimation { duration: 200 }
    }
}
