#include "import_interaction_controller.hpp"

#include "project_view_model.hpp"
#include <QObject>
#include <QState>
#include <QStateMachine>
#include <QString>
#include <QtCore/qtmetamacros.h>
#include <array>
#include <utility>

namespace panta::bridge {

ImportInteractionController::ImportInteractionController(ProjectViewModel* project, QObject* parent)
    : QObject(parent), m_project(project) {
    auto* closedState = new QState(&m_machine);
    auto* editingState = new QState(&m_machine);
    auto* choosingFileState = new QState(&m_machine);
    auto* previewingState = new QState(&m_machine);
    auto* readyState = new QState(&m_machine);
    auto* submittingState = new QState(&m_machine);
    auto* projectionRefreshFailedState = new QState(&m_machine);

    m_machine.setInitialState(closedState);
    closedState->addTransition(this, &ImportInteractionController::openRequested, editingState);
    editingState->addTransition(this, &ImportInteractionController::previewRequested,
                                previewingState);
    editingState->addTransition(this, &ImportInteractionController::chooseFileRequested,
                                choosingFileState);
    readyState->addTransition(this, &ImportInteractionController::chooseFileRequested,
                              choosingFileState);
    choosingFileState->addTransition(this, &ImportInteractionController::fileSelectionAccepted,
                                     previewingState);
    choosingFileState->addTransition(
        this, &ImportInteractionController::fileSelectionCancelledToEditing, editingState);
    choosingFileState->addTransition(
        this, &ImportInteractionController::fileSelectionCancelledToReady, readyState);
    readyState->addTransition(this, &ImportInteractionController::previewRequested,
                              previewingState);
    readyState->addTransition(this, &ImportInteractionController::submitRequested, submittingState);
    previewingState->addTransition(this, &ImportInteractionController::previewSucceeded,
                                   readyState);
    previewingState->addTransition(this, &ImportInteractionController::previewFailed, editingState);
    submittingState->addTransition(this, &ImportInteractionController::submitRejected,
                                   editingState);
    submittingState->addTransition(this, &ImportInteractionController::importCommitted,
                                   closedState);
    submittingState->addTransition(this, &ImportInteractionController::importFailed, editingState);
    submittingState->addTransition(this, &ImportInteractionController::projectionRefreshFailed,
                                   projectionRefreshFailedState);

    for (auto* state : std::array{editingState, choosingFileState, previewingState, readyState,
                                  submittingState, projectionRefreshFailedState}) {
        state->addTransition(this, &ImportInteractionController::closeRequested, closedState);
    }

    connect(closedState, &QState::entered, this, [this] {
        setState(State::Closed);
        if (m_initialStateEntered) {
            m_transitionPending = false;
            m_closeAfterTransition = false;
            m_closeTransitionPending = false;
            m_fileSelectionTransitionPending = false;
            m_fileSelectionDetached = false;
            m_project->cancelImportPreview();
            if (m_project->importPending() && m_project->cancelImport()) {
                m_importRequestId = 0;
            }
            emit closed();
        } else {
            m_initialStateEntered = true;
        }
    });
    connect(editingState, &QState::entered, this, [this] { finishStateEntry(State::Editing); });
    connect(choosingFileState, &QState::entered, this, [this] {
        m_fileSelectionTransitionPending = false;
        const bool detached = std::exchange(m_fileSelectionDetached, false);
        finishStateEntry(State::ChoosingFile);
        if (detached && m_state == State::ChoosingFile && !m_transitionPending) {
            cancelFileSelection(m_fileSelectionGeneration);
        }
    });
    connect(previewingState, &QState::entered, this, [this] {
        finishStateEntry(State::Previewing);
        if (!m_project->inspectStl(m_sourcePath)) {
            emit previewFailed();
            return;
        }
        m_previewRequestId = m_project->importPreviewRequestId();
    });
    connect(readyState, &QState::entered, this, [this] { finishStateEntry(State::Ready); });
    connect(submittingState, &QState::entered, this, [this] {
        finishStateEntry(State::Submitting);
        if (!m_project->beginImportStl(m_sourcePath, m_meshType, m_units, m_showImportLog)) {
            emit submitRejected();
            return;
        }
        m_importRequestId = m_project->importRequestId();
        m_importSessionGeneration = m_sessionGeneration;
    });
    connect(projectionRefreshFailedState, &QState::entered, this,
            [this] { finishStateEntry(State::ProjectionRefreshFailed); });

    connect(&m_machine, &QStateMachine::started, this, [this] {
        m_machineStarted = true;
        startRequestedState();
    });
    connect(m_project, &ProjectViewModel::importPreviewFinished, this,
            &ImportInteractionController::onPreviewFinished);
    connect(m_project, &ProjectViewModel::projectImportFinished, this,
            &ImportInteractionController::onImportFinished);
    connect(m_project, &ProjectViewModel::importPendingChanged, this,
            &ImportInteractionController::stateChanged);
    m_machine.start();
}

QString ImportInteractionController::stateName() const {
    switch (m_state) {
    case State::Closed:
        return QStringLiteral("closed");
    case State::Editing:
        return QStringLiteral("editing");
    case State::ChoosingFile:
        return QStringLiteral("choosing-file");
    case State::Previewing:
        return QStringLiteral("previewing");
    case State::Ready:
        return QStringLiteral("ready");
    case State::Submitting:
        return QStringLiteral("submitting");
    case State::ProjectionRefreshFailed:
        return QStringLiteral("projection-refresh-failed");
    }
    return QStringLiteral("closed");
}

bool ImportInteractionController::canEdit() const {
    return (m_state == State::Editing || m_state == State::Ready) && !m_project->importPending();
}

bool ImportInteractionController::canSubmit() const {
    return m_state == State::Ready && !m_project->importPending();
}

void ImportInteractionController::setMeshType(const QString& value) {
    if (!canEdit() || m_meshType == value) {
        return;
    }
    m_meshType = value;
    emit formChanged();
}

void ImportInteractionController::setUnits(const QString& value) {
    if (!canEdit() || m_units == value) {
        return;
    }
    m_units = value;
    emit formChanged();
}

void ImportInteractionController::setShowImportLog(bool value) {
    if (!canEdit() || m_showImportLog == value) {
        return;
    }
    m_showImportLog = value;
    emit formChanged();
}

bool ImportInteractionController::open() {
    if (m_closeAfterTransition || m_closeTransitionPending) {
        return false;
    }
    if (m_transitionPending) {
        return true;
    }
    // 窗口对象重建时，继续显示已有交互会话，不重置表单或重发服务命令。
    if (m_state != State::Closed) {
        return true;
    }
    m_transitionPending = true;
    ++m_sessionGeneration;
    if (!m_project->importPending()) {
        m_project->clearError();
        m_sourcePath.clear();
        m_meshType = m_project->defaultMeshType();
        m_units = QStringLiteral("millimeters");
        m_showImportLog = true;
        emit formChanged();
    }
    if (!m_machineStarted) {
        return true;
    }
    startRequestedState();
    return true;
}

bool ImportInteractionController::close() {
    if (m_transitionPending) {
        if (!m_closeAfterTransition) {
            m_closeAfterTransition = true;
            ++m_sessionGeneration;
            ++m_fileSelectionGeneration;
        }
        return true;
    }
    if (m_state == State::Closed) {
        return false;
    }
    m_transitionPending = true;
    m_closeTransitionPending = true;
    ++m_sessionGeneration;
    ++m_fileSelectionGeneration;
    emit closeRequested();
    return true;
}

quint64 ImportInteractionController::beginFileSelection() {
    if (!canEdit() || m_transitionPending) {
        return 0;
    }
    m_stateBeforeFileSelection = m_state;
    ++m_fileSelectionGeneration;
    m_fileSelectionTransitionPending = true;
    m_transitionPending = true;
    emit chooseFileRequested();
    return m_fileSelectionGeneration;
}

bool ImportInteractionController::cancelFileSelection(quint64 selectionId) {
    if (selectionId == 0 || selectionId != m_fileSelectionGeneration ||
        m_state != State::ChoosingFile || m_transitionPending) {
        return false;
    }
    m_fileSelectionTransitionPending = false;
    m_transitionPending = true;
    if (m_stateBeforeFileSelection == State::Ready) {
        emit fileSelectionCancelledToReady();
    } else {
        emit fileSelectionCancelledToEditing();
    }
    return true;
}

bool ImportInteractionController::previewFile(const QString& path) {
    if (!canEdit() || m_transitionPending || path.isEmpty()) {
        return false;
    }
    m_fileSelectionTransitionPending = false;
    m_sourcePath = path;
    m_transitionPending = true;
    emit formChanged();
    emit previewRequested();
    return true;
}

bool ImportInteractionController::previewFileForSelection(const QString& path,
                                                          quint64 selectionId) {
    if (selectionId == 0 || selectionId != m_fileSelectionGeneration ||
        m_state != State::ChoosingFile || m_transitionPending || path.isEmpty()) {
        return false;
    }
    m_fileSelectionTransitionPending = false;
    m_sourcePath = path;
    m_transitionPending = true;
    emit formChanged();
    emit fileSelectionAccepted();
    return true;
}

void ImportInteractionController::detachWindow() {
    ++m_fileSelectionGeneration;
    if (m_fileSelectionTransitionPending) {
        m_fileSelectionDetached = true;
        return;
    }
    if (m_state != State::ChoosingFile || m_transitionPending) {
        return;
    }
    m_transitionPending = true;
    if (m_stateBeforeFileSelection == State::Ready) {
        emit fileSelectionCancelledToReady();
    } else {
        emit fileSelectionCancelledToEditing();
    }
}

bool ImportInteractionController::submit() {
    if (!canSubmit() || m_transitionPending) {
        return false;
    }
    m_transitionPending = true;
    emit submitRequested();
    return true;
}

void ImportInteractionController::setState(State state) {
    if (m_state == state) {
        return;
    }
    m_state = state;
    emit stateChanged();
}

void ImportInteractionController::finishStateEntry(State state) {
    m_transitionPending = false;
    setState(state);
    if (m_closeAfterTransition) {
        m_closeAfterTransition = false;
        close();
    }
}

void ImportInteractionController::startRequestedState() {
    if (!m_machineStarted || !m_transitionPending) {
        return;
    }
    if (m_state != State::Closed) {
        return;
    }
    emit openRequested();
}

void ImportInteractionController::onPreviewFinished(quint64 requestId, bool succeeded) {
    if (requestId != m_previewRequestId || m_state != State::Previewing) {
        return;
    }
    m_previewRequestId = 0;
    emit succeeded ? previewSucceeded() : previewFailed();
}

void ImportInteractionController::onImportFinished(quint64 requestId, bool operationSucceeded,
                                                   bool projectionUpdated) {
    if (requestId != m_importRequestId) {
        return;
    }
    m_importRequestId = 0;
    if (m_state != State::Submitting || m_importSessionGeneration != m_sessionGeneration) {
        return;
    }
    if (!operationSucceeded) {
        emit importFailed();
    } else if (projectionUpdated) {
        emit importCommitted();
    } else {
        emit projectionRefreshFailed();
    }
}

} // namespace panta::bridge
