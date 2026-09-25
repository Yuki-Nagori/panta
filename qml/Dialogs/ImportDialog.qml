// STL 预检与导入窗口；文件解析和工程持久化由 ProjectViewModel 完成。
import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Panta.Bridge

Window {
    id: dialog
    objectName: "importDialog"

    required property ProjectViewModel projectModel
    property Window ownerWindow
    property string sourcePath: ""

    width: 680
    height: 430
    minimumWidth: 580
    minimumHeight: 360
    color: "transparent"
    modality: Qt.ApplicationModal
    flags: Qt.Dialog | Qt.FramelessWindowHint
    title: qsTranslate("UiCommon", "Import")
    transientParent: ownerWindow

    readonly property var meshTypeValues: ["midplane", "dual-domain", "solid-3d"]
    readonly property var unitValues: ["millimeters", "centimeters", "inches"]

    function resetFields() {
        projectModel.clearError();
        sourcePath = "";
        meshTypeCombo.currentIndex = 1;
        unitsCombo.currentIndex = 0;
        showImportLogCheckBox.checked = true;
        helpText.visible = false;
    }

    function open() {
        resetFields();
        if (ownerWindow) {
            x = ownerWindow.x + Math.round((ownerWindow.width - width) / 2);
            y = ownerWindow.y + Math.round((ownerWindow.height - height) / 2);
        }
        visible = true;
        requestActivate();
    }

    function chooseFile() {
        fileDialog.open();
    }

    function submit() {
        if (!sourcePath || !projectModel.importPreviewReady)
            return;
        if (projectModel.importStl(sourcePath, meshTypeValues[meshTypeCombo.currentIndex], unitValues[unitsCombo.currentIndex], showImportLogCheckBox.checked)) {
            dialog.close();
        }
    }

    function errorMessage() {
        if (projectModel.errorCode === "project.import_file_missing")
            return qsTranslate("ImportDialogErrors", "The selected STL file does not exist.");
        if (projectModel.errorCode === "project.import_invalid_file")
            return qsTranslate("ImportDialogErrors", "Choose an STL file to import.");
        if (projectModel.errorCode === "project.import_parse_failed")
            return qsTranslate("ImportDialogErrors", "The selected STL file could not be read.");
        if (projectModel.errorCode === "project.import_asset_copy_failed")
            return qsTranslate("ImportDialogErrors", "The STL file could not be copied into the project.");
        return qsTranslate("ImportDialogErrors", "The import operation could not be completed.");
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.colorPanel

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            DialogTitleBar {
                Layout.fillWidth: true
                window: dialog
                caption: dialog.title
                onCloseRequested: dialog.close()
            }

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
                        ThemedToolButton {
                            Layout.preferredWidth: 92
                            Layout.preferredHeight: Theme.controlHeight
                            text: qsTranslate("UiCommonNavigation", "Browse")
                            iconName: "document-open"
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
                    ComboBox {
                        id: meshTypeCombo
                        Layout.fillWidth: true
                        model: [qsTranslate("ImportMeshMidplane", "Midplane"), qsTranslate("ImportMeshDualDomain", "Dual Domain"), qsTranslate("ImportMeshSolid3D", "Solid 3D")]
                        Accessible.name: qsTranslate("ImportDialogForm", "Mesh type")
                    }

                    ThemedLabel {
                        text: qsTranslate("ImportDialogForm", "Units")
                        textSize: Theme.fontBody
                    }
                    ComboBox {
                        id: unitsCombo
                        Layout.fillWidth: true
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

                CheckBox {
                    id: showImportLogCheckBox
                    text: qsTranslate("ImportDialogForm", "Show import log")
                    Layout.fillWidth: true
                }

                ThemedLabel {
                    id: helpText
                    Layout.fillWidth: true
                    visible: false
                    text: qsTranslate("ImportDialogHelp", "The source file is copied into the project assets so the import can be reopened later.")
                    textSize: Theme.fontSmall
                    textColor: Theme.colorTextMuted
                    wrapMode: Text.Wrap
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

                    ThemedToolButton {
                        Layout.preferredWidth: 92
                        Layout.preferredHeight: Theme.controlHeight
                        text: qsTranslate("UiCommonHelp", "Help")
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: helpText.visible = !helpText.visible
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    ThemedToolButton {
                        Layout.preferredWidth: 92
                        Layout.preferredHeight: Theme.controlHeight
                        text: qsTranslate("DialogAction", "OK")
                        highlighted: true
                        enabled: dialog.sourcePath.length > 0 && dialog.projectModel.importPreviewReady
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: dialog.submit()
                    }
                    ThemedToolButton {
                        Layout.preferredWidth: 92
                        Layout.preferredHeight: Theme.controlHeight
                        text: qsTranslate("DialogAction", "Cancel")
                        contentColor: Theme.colorText
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: dialog.close()
                    }
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
            const selectedPath = dialog.projectModel.localPath(selectedFile);
            if (!selectedPath)
                return;
            dialog.sourcePath = selectedPath;
            dialog.projectModel.inspectStl(selectedPath);
        }
    }
}
