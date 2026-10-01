// 独立的新建工程窗口；工程命令通过显式 ProjectViewModel 注入。
import QtQuick
import QtQuick.Dialogs
import QtQuick.Layouts
import Panta.Bridge

DialogWindow {
    id: dialog
    objectName: "newProjectDialog"

    required property ProjectViewModel projectModel
    signal projectCreated(string path)

    width: Theme.newProjectDialogWidth
    height: Theme.newProjectDialogHeight
    minimumWidth: Theme.newProjectDialogMinimumWidth
    minimumHeight: Theme.newProjectDialogMinimumHeight
    title: qsTranslate("NewProjectDialog", "Create New Project")

    function resetFields() {
        projectModel.clearError();
        projectNameField.text = "";
        locationField.text = projectModel.defaultLocation;
    }

    function open() {
        resetFields();
        centerOnOwner();
        visible = true;
        requestActivate();
        projectNameField.forceActiveFocus();
    }

    function submit() {
        if (projectModel.createProject(projectNameField.text, locationField.text)) {
            const path = projectModel.lastCreatedPath;
            dialog.projectCreated(path);
            dialog.close();
        } else {
            projectNameField.forceActiveFocus();
        }
    }

    function errorMessage() {
        if (projectModel.errorCode === "project.empty_name" || projectModel.errorCode === "project.invalid_name") {
            return qsTranslate("NewProjectDialog", "Enter a valid project name.");
        }
        if (projectModel.errorCode === "project.location_empty" || projectModel.errorCode === "project.location_not_absolute") {
            return qsTranslate("NewProjectDialog", "Choose an absolute project location.");
        }
        if (projectModel.errorCode === "project.location_create_failed") {
            return qsTranslate("NewProjectDialog", "The project location could not be created.");
        }
        if (projectModel.errorCode === "project.already_exists") {
            return qsTranslate("NewProjectDialog", "A project with this name already exists.");
        }
        if (projectModel.errorCode === "project.manifest_invalid" || projectModel.errorCode === "project.unsupported_schema") {
            return qsTranslate("NewProjectDialog", "The selected folder is not a supported panta project.");
        }
        if (projectModel.errorCode === "project.no_project") {
            return qsTranslate("NewProjectDialog", "Open or create a project first.");
        }
        if (projectModel.errorCode === "project.command_invalid") {
            return qsTranslate("NewProjectDialog", "The project change is not valid.");
        }
        return qsTranslate("NewProjectDialog", "The project operation could not be completed.");
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
                text: qsTranslate("NewProjectDialog", "Create a project directory and start with an empty workspace.")
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
                    text: qsTranslate("NewProjectDialog", "Project name")
                    textSize: Theme.fontBody
                }
                ThemedTextField {
                    id: projectNameField
                    objectName: "projectNameField"
                    Layout.fillWidth: true
                    Layout.preferredHeight: Theme.controlHeight
                    placeholderText: qsTranslate("NewProjectDialog", "My Project")
                    Accessible.name: qsTranslate("NewProjectDialog", "Project name")
                    invalid: dialog.projectModel.errorCode === "project.empty_name" || dialog.projectModel.errorCode === "project.invalid_name"
                    onAccepted: dialog.submit()
                }

                ThemedLabel {
                    text: qsTranslate("NewProjectDialog", "Create in")
                    textSize: Theme.fontBody
                }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingSmall

                    ThemedTextField {
                        id: locationField
                        objectName: "projectLocationField"
                        Layout.fillWidth: true
                        Layout.preferredHeight: Theme.controlHeight
                        placeholderText: qsTranslate("NewProjectDialog", "Choose a folder")
                        Accessible.name: qsTranslate("NewProjectDialog", "Project location")
                        selectByMouse: true
                        invalid: dialog.projectModel.errorCode === "project.location_empty" || dialog.projectModel.errorCode === "project.location_not_absolute"
                    }
                    ThemedButton {
                        text: qsTranslate("UiCommonNavigation", "Browse")
                        iconName: "open"
                        preserveIconColors: true
                        contentPadding: Theme.spacingSmall
                        hoverColor: Theme.colorHover
                        borderColor: Theme.colorPanelLine
                        onClicked: folderDialog.open()
                    }
                }
            }

            ThemedLabel {
                Layout.fillWidth: true
                visible: dialog.projectModel.error.length > 0
                text: dialog.errorMessage()
                textSize: Theme.fontSmall
                textColor: Theme.colorError
                wrapMode: Text.Wrap
            }

            ThemedLabel {
                Layout.fillWidth: true
                text: qsTranslate("NewProjectDialog", "The project folder and .panta file will be created if they do not already exist.")
                textSize: Theme.fontSmall
                textColor: Theme.colorTextMuted
                wrapMode: Text.Wrap
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }

            DialogButtonRow {
                Layout.fillWidth: true
                spacing: Theme.spacingSmall

                ThemedButton {
                    text: qsTranslate("DialogAction", "OK")
                    primaryAction: true
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

    FolderDialog {
        id: folderDialog
        title: qsTranslate("NewProjectDialog", "Choose Project Location")
        currentFolder: dialog.projectModel.defaultLocationUrl
        onAccepted: {
            const selectedPath = dialog.projectModel.localPath(selectedFolder);
            if (selectedPath.length > 0) {
                locationField.text = selectedPath;
                dialog.projectModel.clearError();
            }
        }
    }

    Shortcut {
        sequence: "Esc"
        onActivated: dialog.close()
    }
}
