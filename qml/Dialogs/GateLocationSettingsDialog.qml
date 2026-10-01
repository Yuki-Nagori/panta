// 浇口定位工艺候选；Rust 后台确认成功后由宿主关闭。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

DialogWindow {
    id: dialog
    objectName: "gateLocationSettingsDialog"
    property var planSettings: ({})
    property var openedSettings: ({})
    property bool saving: false
    property string errorText: ""
    signal settingsRequested(string projectPath, double revision, string importId, var settings)
    readonly property bool canConfirm: !saving && !!openedSettings.importId && moldTemperature.acceptableInput && meltTemperature.acceptableInput && numberOfGates.acceptableInput
    width: Theme.gateLocationSettingsDialogWidth
    height: Theme.gateLocationSettingsDialogHeight
    minimumWidth: Theme.gateLocationSettingsDialogMinimumWidth
    minimumHeight: Theme.gateLocationSettingsDialogMinimumHeight
    title: qsTranslate("GateLocationSettings", "Process Settings · Gate Location")
    onClosing: event => {
        if (saving)
            event.accepted = false;
    }

    function open() {
        if (saving || planSettings.sequenceId !== "gate-location" || !planSettings.importId)
            return;
        openedSettings = planSettings;
        moldTemperature.text = String(openedSettings.gateLocationSettings.moldTemperature);
        meltTemperature.text = String(openedSettings.gateLocationSettings.meltTemperature);
        numberOfGates.text = String(openedSettings.gateLocationSettings.numberOfGates);
        centerOnOwner();
        show();
        requestActivate();
        moldTemperature.forceActiveFocus();
    }
    function acceptSettings() {
        if (!visible || !canConfirm)
            return;
        const settings = openedSettings.gateLocationSettings;
        settingsRequested(openedSettings.projectPath, openedSettings.revision, openedSettings.importId, {
            machineId: settings.machineId,
            algorithmId: settings.algorithmId,
            moldTemperature: Number(moldTemperature.text),
            meltTemperature: Number(meltTemperature.text),
            numberOfGates: Number(numberOfGates.text)
        });
    }
    DialogFrame {
        window: dialog
        closeEnabled: true

        ThemedLabel {
            Layout.fillWidth: true
            Layout.margins: Theme.spacingLarge
            visible: dialog.saving || dialog.errorText.length > 0
            text: dialog.saving ? qsTranslate("FillSettings", "Saving process settings…") : dialog.errorText
            textColor: dialog.saving ? Theme.colorTextMuted : Theme.colorError
            wrapMode: Text.Wrap
        }
        ScrollView {
            id: gateSettingsScroll
            enabled: !dialog.saving
            Layout.fillWidth: true
            Layout.fillHeight: true
            contentWidth: availableWidth
            clip: true
            ColumnLayout {
                width: gateSettingsScroll.availableWidth
                spacing: Theme.spacingLarge
                FormSection {
                    Layout.margins: Theme.spacingLarge
                    title: qsTranslate("GateLocationSettings", "Injection molding machine")
                    RowLayout {
                        anchors.fill: parent
                        ThemedComboBox {
                            Layout.fillWidth: true

                            font.pixelSize: Theme.fontBody
                            model: [qsTranslate("GateLocationSettings", dialog.openedSettings.gateLocationSettings?.machineSourceText ?? "")]
                            Accessible.name: qsTranslate("GateLocationSettings", "Injection molding machine")
                        }
                        ThemedButton {
                            contentPadding: Theme.spacingLarge
                            implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                            text: qsTranslate("GateLocationSettings", "Edit…")
                            enabled: false
                        }
                        ThemedButton {
                            contentPadding: Theme.spacingLarge
                            implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                            text: qsTranslate("GateLocationSettings", "Select…")
                            enabled: false
                        }
                    }
                }
                FormSection {
                    Layout.leftMargin: Theme.spacingLarge
                    Layout.rightMargin: Theme.spacingLarge
                    title: qsTranslate("FillSettings", "Temperatures")
                    GridLayout {
                        anchors.fill: parent
                        columns: 2
                        uniformCellWidths: true
                        columnSpacing: Theme.spacingLarge
                        ThemedLabel {
                            text: qsTranslate("FillSettings", "Mold surface temperature")
                        }
                        ThemedLabel {
                            text: qsTranslate("FillSettings", "Melt temperature")
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Layout.preferredWidth: 0
                            ThemedNumberField {
                                id: moldTemperature
                                Layout.fillWidth: true
                                objectName: "gateMoldTemperature"
                                Accessible.name: qsTranslate("FillSettings", "Mold surface temperature")
                                validator: DoubleValidator {
                                    bottom: -273.15
                                    locale: "C"
                                    notation: DoubleValidator.StandardNotation
                                }
                            }
                            ThemedLabel {
                                text: "°C"
                                textColor: Theme.colorTextMuted
                            }
                        }
                        RowLayout {
                            Layout.fillWidth: true
                            Layout.preferredWidth: 0
                            ThemedNumberField {
                                id: meltTemperature
                                Layout.fillWidth: true
                                objectName: "gateMeltTemperature"
                                Accessible.name: qsTranslate("FillSettings", "Melt temperature")
                                validator: DoubleValidator {
                                    bottom: -273.15
                                    locale: "C"
                                    notation: DoubleValidator.StandardNotation
                                }
                            }
                            ThemedLabel {
                                text: "°C"
                                textColor: Theme.colorTextMuted
                            }
                        }
                    }
                }
                FormSection {
                    Layout.leftMargin: Theme.spacingLarge
                    Layout.rightMargin: Theme.spacingLarge
                    Layout.bottomMargin: Theme.spacingLarge
                    title: qsTranslate("GateLocationSettings", "Gate locator")
                    ColumnLayout {
                        anchors.fill: parent
                        spacing: Theme.spacingLarge
                        GridLayout {
                            columns: 2
                            Layout.fillWidth: true
                            columnSpacing: Theme.spacingLarge
                            ThemedLabel {
                                text: qsTranslate("GateLocationSettings", "Algorithm")
                            }
                            ThemedLabel {
                                text: qsTranslate("GateLocationSettings", "Number of gates")
                            }
                            ThemedComboBox {
                                Layout.fillWidth: true

                                font.pixelSize: Theme.fontBody
                                model: [qsTranslate("GateLocationSettings", dialog.openedSettings.gateLocationSettings?.algorithmSourceText ?? "")]
                                Accessible.name: qsTranslate("GateLocationSettings", "Algorithm")
                            }
                            RowLayout {
                                Layout.preferredWidth: Theme.processValueColumnWidth
                                ThemedNumberField {
                                    id: numberOfGates
                                    Layout.fillWidth: true
                                    objectName: "gateNumberOfGates"
                                    Accessible.name: qsTranslate("GateLocationSettings", "Number of gates")
                                    validator: IntValidator {
                                        bottom: 1
                                        top: 10
                                        locale: "C"
                                    }
                                }
                                ThemedLabel {
                                    text: "1–10"
                                    textColor: Theme.colorTextMuted
                                }
                            }
                        }
                        ThemedButton {
                            contentPadding: Theme.spacingLarge
                            implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)
                            text: qsTranslate("FillSettings", "Advanced Options…")
                            enabled: false
                        }
                    }
                }
            }
        }
        DialogButtonRow {
            Layout.fillWidth: true
            Layout.margins: Theme.spacingLarge

            ThemedButton {
                contentPadding: Theme.spacingLarge
                objectName: "gateLocationSettingsAccept"

                text: qsTranslate("DialogAction", "OK")
                enabled: dialog.canConfirm
                primaryAction: true
                onClicked: dialog.acceptSettings()
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge
                objectName: "gateLocationSettingsCancel"

                text: qsTranslate("DialogAction", "Cancel")
                enabled: !dialog.saving
                onClicked: dialog.close()
            }
            ThemedButton {
                contentPadding: Theme.spacingLarge
                implicitWidth: Math.max(Theme.dialogActionWidth, contentItem.implicitWidth + leftPadding + rightPadding)

                text: qsTranslate("UiCommonHelp", "Help")
                enabled: false
            }
        }
    }
    Shortcut {
        sequence: "Esc"
        enabled: dialog.visible && !dialog.saving
        onActivated: dialog.close()
    }
}
