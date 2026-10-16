// 导入窗口只提交意图；预检由 Rust 后台完成，工程提交由服务持有。
import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import Panta.Bridge

DialogWindow {
    id: dialog
    objectName: "importDialog"

    required property ProjectViewModel projectModel
    property string sourcePath: ""
    property var pendingImportRequestId: 0

    width: Theme.importDialogWidth
    height: Theme.importDialogHeight
    minimumWidth: Theme.importDialogMinimumWidth
    minimumHeight: Theme.importDialogMinimumHeight
    title: qsTranslate("UiCommon", "Import")

    readonly property var meshTypes: projectModel.meshTypes
    readonly property var unitValues: ["millimeters", "centimeters", "inches"]
    // A surface sidecar uses 72 bytes per triangle plus a 64-byte header.
    readonly property bool largeMeshWarning: projectModel.importPreviewReady && projectModel.importPreviewTriangleCount * 72 + 64 > 2147483648

    onVisibleChanged: {
        if (!visible) {
            pendingImportRequestId = 0;
            projectModel.cancelImportPreview();
            projectModel.cancelImport();
        }
    }

    function resetFields() {
        projectModel.cancelImportPreview();
        projectModel.clearError();
        sourcePath = "";
        meshTypeCombo.currentIndex = meshTypes.findIndex(entry => entry.id === projectModel.defaultMeshType);
        unitsCombo.currentIndex = 0;
        showImportLogCheckBox.checked = true;
        helpText.visible = false;
    }

    function open() {
        resetFields();
        pendingImportRequestId = 0;
        centerOnOwner();
        visible = true;
        requestActivate();
    }

    function chooseFile() {
        fileDialog.open();
    }

    function submit() {
        if (!sourcePath || !projectModel.importPreviewReady)
            return;
        if (projectModel.beginImportStl(sourcePath, meshTypes[meshTypeCombo.currentIndex].id, unitValues[unitsCombo.currentIndex], showImportLogCheckBox.checked))
            pendingImportRequestId = projectModel.importRequestId;
    }

    function errorMessage() {
        if (projectModel.errorCode === "project.import_file_missing")
            return qsTranslate("ImportDialogErrors", "The selected STL file does not exist.");
        if (projectModel.errorCode === "project.import_invalid_file")
            return qsTranslate("ImportDialogErrors", "Choose an STL file to import.");
        if (projectModel.errorCode === "project.import_parse_failed")
            return qsTranslate("ImportDialogErrors", "The selected STL file could not be read.");
        if (projectModel.errorCode === "project.import_asset_write_failed")
            return qsTranslate("ImportDialogErrors", "The mesh asset could not be saved into the project.");
        return qsTranslate("ImportDialogErrors", "The import operation could not be completed.");
    }

    DialogFrame {
        window: dialog
        closeEnabled: true

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.leftMargin: Theme.spacingLarge
            Layout.rightMargin: Theme.spacingLarge
            Layout.topMargin: Theme.spacingMedium
            Layout.bottomMargin: Theme.spacingMedium
            spacing: Theme.spacingMedium

            ThemedLabel {
                Layout.fillWidth: true
                text: qsTranslate("ImportDialogForm", "Select a mesh type and units before importing the STL file.")
                textSize: Theme.fontSmall
                textColor: Theme.colorTextMuted
                wrapMode: Text.Wrap
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 2
                columnSpacing: Theme.spacingMedium
                rowSpacing: Theme.spacingMedium

                ThemedLabel {
                    text: qsTranslate("UiCommon", "File")
                    textSize: Theme.fontBody
                }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingSmall

                    ThemedTextField {
                        id: sourceField
                        Layout.fillWidth: true
                        Layout.preferredHeight: Theme.controlHeight
                        text: dialog.sourcePath
                        readOnly: true
                        placeholderText: qsTranslate("ImportDialogForm", "Choose an STL file")
                        Accessible.name: qsTranslate("ImportDialogForm", "STL file")
                    }
                    ThemedButton {
                        text: qsTranslate("UiCommonNavigation", "Browse")
                        enabled: !dialog.projectModel.importPending
                        iconName: "open"
                        preserveIconColors: true
                        contentPadding: Theme.spacingSmall
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: dialog.chooseFile()
                    }
                }

                ThemedLabel {
                    text: qsTranslate("ImportDialogForm", "Mesh type")
                    textSize: Theme.fontBody
                }
                ThemedComboBox {
                    id: meshTypeCombo
                    Layout.fillWidth: true
                    enabled: !dialog.projectModel.importPending
                    model: dialog.meshTypes.map(entry => qsTranslate("MeshType", entry.sourceText))
                    Accessible.name: qsTranslate("ImportDialogForm", "Mesh type")
                }

                ThemedLabel {
                    text: qsTranslate("ImportDialogForm", "Units")
                    textSize: Theme.fontBody
                }
                ThemedComboBox {
                    id: unitsCombo
                    Layout.fillWidth: true
                    enabled: !dialog.projectModel.importPending
                    model: [qsTranslate("UiCommonUnitMillimeters", "Millimeters"), qsTranslate("UiCommonUnitCentimeters", "Centimeters"), qsTranslate("UiCommonUnitInches", "Inches")]
                    Accessible.name: qsTranslate("ImportDialogForm", "Units")
                }

                ThemedLabel {
                    text: qsTranslate("ImportDialogForm", "Approximate dimensions")
                    textSize: Theme.fontBody
                }
                ThemedLabel {
                    Layout.fillWidth: true
                    text: dialog.projectModel.importPreviewReady && dialog.sourcePath.length > 0 ? dialog.projectModel.importPreviewDimensions + " " + unitsCombo.currentText.toLowerCase() : qsTranslate("ImportDialogStatus", "Select an STL file")
                    textSize: Theme.fontBody
                    textColor: Theme.colorTextMuted
                }
            }

            ThemedLabel {
                Layout.fillWidth: true
                visible: dialog.largeMeshWarning
                text: qsTranslate("ImportDialogStatus", "The expanded mesh data is estimated to exceed 2 GiB. Import is allowed, but it may take a long time and require substantial memory and disk space. Make sure your computer and project drive have enough resources before continuing.")
                textSize: Theme.fontSmall
                textColor: Theme.colorError
                wrapMode: Text.Wrap
            }

            ThemedCheckBox {
                id: showImportLogCheckBox

                text: qsTranslate("ImportDialogForm", "Show import log")
                Layout.fillWidth: true
                enabled: !dialog.projectModel.importPending
            }

            ThemedLabel {
                id: helpText
                Layout.fillWidth: true
                visible: false
                text: qsTranslate("ImportDialogHelp", "A mesh asset is saved into the project. The original STL can be moved or deleted after import.")
                textSize: Theme.fontSmall
                textColor: Theme.colorTextMuted
                wrapMode: Text.Wrap
            }

            ThemedLabel {
                Layout.fillWidth: true
                visible: dialog.projectModel.importPending
                text: qsTranslate("ImportDialogStatus", "Adding the STL file to the project…")
                textSize: Theme.fontSmall
                textColor: Theme.colorTextMuted
            }

            ThemedLabel {
                Layout.fillWidth: true
                visible: dialog.projectModel.error.length > 0
                text: dialog.errorMessage()
                textSize: Theme.fontSmall
                textColor: Theme.colorError
                wrapMode: Text.Wrap
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingSmall

                ThemedButton {
                    text: qsTranslate("UiCommonHelp", "Help")
                    enabled: !dialog.projectModel.importPending
                    contentColor: Theme.colorText
                    hoverColor: Theme.colorHover
                    borderColor: Theme.colorPanelLine
                    onClicked: helpText.visible = !helpText.visible
                }

                Item {
                    Layout.fillWidth: true
                }

                ThemedButton {
                    text: dialog.projectModel.importPending ? qsTranslate("ImportDialogStatus", "Importing…") : qsTranslate("DialogAction", "OK")
                    objectName: "importAccept"
                    primaryAction: true
                    enabled: dialog.sourcePath.length > 0 && dialog.projectModel.importPreviewReady && !dialog.projectModel.importPending
                    contentColor: Theme.colorText
                    hoverColor: Theme.colorHover
                    onClicked: dialog.submit()
                }
                ThemedButton {
                    text: qsTranslate("DialogAction", "Cancel")
                    contentColor: Theme.colorText
                    hoverColor: Theme.colorHover
                    borderColor: Theme.colorPanelLine
                    onClicked: dialog.close()
                }
            }
        }
    }

    FileDialog {
        id: fileDialog
        title: qsTranslate("ImportDialogFile", "Choose STL File")
        fileMode: FileDialog.OpenFile
        nameFilters: [qsTranslate("ImportDialogFile", "STL files (*.stl)")]
        onAccepted: {
            if (!dialog.visible)
                return;
            const selectedPath = dialog.projectModel.localPath(selectedFile);
            if (!selectedPath)
                return;
            dialog.sourcePath = selectedPath;
            dialog.projectModel.inspectStl(selectedPath);
        }
    }

    Connections {
        target: dialog.projectModel
        function onProjectImportFinished(requestId, succeeded) {
            if (succeeded && dialog.visible && requestId === dialog.pendingImportRequestId) {
                dialog.pendingImportRequestId = 0;
                dialog.close();
            }
        }
    }
}
