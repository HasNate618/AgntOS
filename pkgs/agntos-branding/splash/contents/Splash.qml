import QtQuick 2.15

Rectangle {
    color: "#141416"
    
    Image {
        source: "images/background.png"
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop
    }
    
    Image {
        source: "images/logo.png"
        anchors.centerIn: parent
        width: Math.min(parent.width, parent.height) * 0.3
        fillMode: Image.PreserveAspectFit
    }
}
