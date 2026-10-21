// 浇口定位工艺候选；Rust 后台确认成功后由宿主关闭。
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

DialogWindow {
    id: dialog
    objectName: "gateLocationSettingsDialog"
    property var planSettings: ({})
    property var validationModel: null
    property var openedSettings: ({})
    property bool saving: false
    property string errorText: ""
    escapeEnabled: !saving
    signal settingsRequested(string projectPath, double revision, string importId, var settings)
    readonly property var constraints: planSettings.processConstraints ?? ({})
    readonly property var draftValidation: validationModel ? validationModel.validateGateLocationSettingsDraft({
        machineId: openedSettings.gateLocationSettings?.machineId ?? "",
        moldTemperature: draftNumber(moldTemperature.text),
        meltTemperature: draftNumber(meltTemperature.text),
        numberOfGates: draftNumber(numberOfGates.text)
    }) : ({
            valid: false,
            errors: []
        })
    readonly property bool canConfirm: !saving && !!openedSettings.importId && draftValidation.valid && moldTemperature.acceptableInput && meltTemperature.acceptableInput && numberOfGates.acceptableInput
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
        present();
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
    function draftNumber(text) {
        return text.trim().length === 0 ? NaN : Number(text);
    }
    function minimumBound(constraint, fallback) {
        return constraint?.hasMinimum ? constraint.minimum : fallback;
    }
    function maximumBound(constraint, fallback) {
        return constraint?.hasMaximum ? constraint.maximum : fallback;
    }
    function rangeText(constraint) {
        if (!constraint)
            return "";
        if (constraint.hasMinimum && constraint.minimumInclusive && constraint.hasMaximum && constraint.maximumInclusive)
            return "%1–%2".arg(constraint.minimum).arg(constraint.maximum);
        const minimum = constraint.hasMinimum ? (constraint.minimumInclusive ? "≥ " : "> ") + constraint.minimum : "";
        const maximum = constraint.hasMaximum ? (constraint.maximumInclusive ? " ≤ " : " < ") + constraint.maximum : "";
        return minimum + maximum;
    }
    DialogFrame {
        window: dialog
        closeEnabled: !dialog.saving

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
                                invalid: !acceptableInput || dialog.draftValidation.errors.indexOf("moldTemperature") >= 0
                                validator: DoubleValidator {
                                    bottom: dialog.minimumBound(dialog.constraints.temperature, -Number.MAX_VALUE)
                                    top: dialog.maximumBound(dialog.constraints.temperature, Number.MAX_VALUE)
                                    locale: "C"
                                    notation: DoubleValidator.StandardNotation
                                }
                            }
                            ThemedLabel {
                                text: dialog.constraints.temperature?.unit ?? ""
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
                                invalid: !acceptableInput || dialog.draftValidation.errors.indexOf("meltTemperature") >= 0
                                validator: DoubleValidator {
                                    bottom: dialog.minimumBound(dialog.constraints.temperature, -Number.MAX_VALUE)
                                    top: dialog.maximumBound(dialog.constraints.temperature, Number.MAX_VALUE)
                                    locale: "C"
                                    notation: DoubleValidator.StandardNotation
                                }
                            }
                            ThemedLabel {
                                text: dialog.constraints.temperature?.unit ?? ""
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
                                    invalid: !acceptableInput || dialog.draftValidation.errors.indexOf("numberOfGates") >= 0
                                    validator: IntValidator {
                                        bottom: dialog.minimumBound(dialog.constraints.numberOfGates, -2147483648)
                                        top: dialog.maximumBound(dialog.constraints.numberOfGates, 2147483647)
                                        locale: "C"
                                    }
                                }
                                ThemedLabel {
                                    text: dialog.rangeText(dialog.constraints.numberOfGates)
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
}
