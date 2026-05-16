import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import org.kde.kirigami 2.20 as Kirigami

Kirigami.ScrollablePage {
    id: root
    title: "Activity"

    property var auditModel: null
    property var onSearch: null
    property var onRollback: null
    signal refreshRequested()

    header: Kirigami.SearchField {
        id: searchField
        placeholderText: "Search audit log..."
        onAccepted: {
            if (root.onSearch) root.onSearch(searchField.text)
        }
        onClearClicked: {
            searchField.text = ""
            root.refreshRequested()
        }
    }

    actions: [
        Kirigami.Action {
            icon.name: "view-refresh"
            tooltip: "Refresh audit log"
            onTriggered: root.refreshRequested()
        }
    ]

    Kirigami.CardsListView {
        model: auditModel
        delegate: AuditEntryCard {
            auditId: model.auditId || ""
            timestamp: model.timestamp || ""
            actionType: model.actionType || ""
            summary: model.summary || ""
            entryStatus: model.status || "unknown"
            actor: model.actor || ""
            prompt: model.prompt || ""
            rationale: model.rationale || ""
            filesChanged: model.filesChanged || []
            rollbackHint: model.rollbackHint || ""
            resultMessage: model.resultMessage || ""

            onRollbackClicked: {
                if (root.onRollback) root.onRollback(auditId)
            }
        }
    }
}
