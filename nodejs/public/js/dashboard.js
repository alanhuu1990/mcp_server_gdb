class DebugDashboard {
    constructor() {
        this.socket = null;
        this.currentSession = null;
        this.sessions = new Map();
        this.autoRefreshVars = true;
        this.autoRefreshRegs = true;
        this.autoScrollLogs = true;
        
        this.initializeSocket();
        this.setupEventListeners();
        this.setupModals();
    }

    initializeSocket() {
        this.socket = io();
        
        this.socket.on('connect', () => {
            console.log('Connected to server');
            this.updateConnectionStatus(true);
            this.addLogEntry('info', 'Connected to debug server');
            
            // Subscribe to all event types
            this.socket.emit('subscribe', { type: 'variables' });
            this.socket.emit('subscribe', { type: 'registers' });
            this.socket.emit('subscribe', { type: 'logs' });
            
            // Request initial data
            this.refreshSessions();
        });

        this.socket.on('disconnect', () => {
            console.log('Disconnected from server');
            this.updateConnectionStatus(false);
            this.addLogEntry('error', 'Disconnected from debug server');
        });

        this.socket.on('sessions_data', (sessions) => {
            this.updateSessions(sessions);
        });

        this.socket.on('variables_data', (data) => {
            if (data.sessionId === this.currentSession) {
                this.updateVariables(data.variables);
            }
        });

        this.socket.on('registers_data', (data) => {
            if (data.sessionId === this.currentSession) {
                this.updateRegisters(data.registers);
            }
        });

        this.socket.on('session_created', (session) => {
            this.addLogEntry('info', `New session created: ${session.name || session.id}`);
            this.refreshSessions();
        });

        this.socket.on('breakpoint_hit', (data) => {
            this.addLogEntry('warn', `Breakpoint hit in session ${data.sessionId}`);
            if (data.sessionId === this.currentSession) {
                this.refreshCurrentSessionData();
            }
        });

        this.socket.on('execution_stopped', (data) => {
            this.addLogEntry('info', `Execution stopped in session ${data.sessionId}`);
            this.updateDebugControls(true);
        });

        this.socket.on('execution_continued', (data) => {
            this.addLogEntry('info', `Execution continued in session ${data.sessionId}`);
            this.updateDebugControls(false);
        });

        this.socket.on('log_message', (logEntry) => {
            this.addLogEntry(logEntry.level, logEntry.message, logEntry.timestamp);
        });

        this.socket.on('error', (error) => {
            console.error('Socket error:', error);
            this.addLogEntry('error', `Error: ${error.message}`);
        });
    }

    setupEventListeners() {
        // Debug control buttons
        document.getElementById('btn-continue').addEventListener('click', () => {
            this.executeDebugCommand('continue');
        });

        document.getElementById('btn-step').addEventListener('click', () => {
            this.executeDebugCommand('step');
        });

        document.getElementById('btn-stop').addEventListener('click', () => {
            this.executeDebugCommand('stop');
        });

        document.getElementById('btn-refresh').addEventListener('click', () => {
            this.refreshCurrentSessionData();
        });

        // Session management
        document.getElementById('btn-new-session').addEventListener('click', () => {
            this.showModal('new-session-modal');
        });

        // Breakpoint management
        document.getElementById('btn-add-breakpoint').addEventListener('click', () => {
            this.showModal('add-breakpoint-modal');
        });

        // Auto-refresh toggles
        document.getElementById('auto-refresh-vars').addEventListener('change', (e) => {
            this.autoRefreshVars = e.target.checked;
        });

        document.getElementById('auto-refresh-regs').addEventListener('change', (e) => {
            this.autoRefreshRegs = e.target.checked;
        });

        document.getElementById('auto-scroll-logs').addEventListener('change', (e) => {
            this.autoScrollLogs = e.target.checked;
        });

        // Clear logs
        document.getElementById('btn-clear-logs').addEventListener('click', () => {
            document.getElementById('logs-content').innerHTML = '';
        });

        // Form submissions
        document.getElementById('new-session-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.createNewSession();
        });

        document.getElementById('add-breakpoint-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.addBreakpoint();
        });
    }

    setupModals() {
        // Modal close handlers
        document.querySelectorAll('.modal-close, #modal-cancel, #bp-modal-cancel').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const modal = e.target.closest('.modal');
                if (modal) {
                    this.hideModal(modal.id);
                }
            });
        });

        // Click outside to close
        document.querySelectorAll('.modal').forEach(modal => {
            modal.addEventListener('click', (e) => {
                if (e.target === modal) {
                    this.hideModal(modal.id);
                }
            });
        });
    }

    updateConnectionStatus(connected) {
        const indicator = document.getElementById('connection-indicator');
        if (connected) {
            indicator.className = 'status-indicator connected';
            indicator.innerHTML = '<i class="fas fa-circle"></i> Connected';
        } else {
            indicator.className = 'status-indicator disconnected';
            indicator.innerHTML = '<i class="fas fa-circle"></i> Disconnected';
        }
    }

    updateSessions(sessions) {
        const sessionsList = document.getElementById('sessions-list');
        
        if (!sessions || sessions.length === 0) {
            sessionsList.innerHTML = '<div class="no-sessions">No active sessions</div>';
            return;
        }

        sessionsList.innerHTML = sessions.map(session => `
            <div class="session-item ${session.id === this.currentSession ? 'active' : ''}" 
                 data-session-id="${session.id}">
                <div class="session-name">${session.name || session.id}</div>
                <div class="session-status">${session.status || 'Unknown'}</div>
            </div>
        `).join('');

        // Add click handlers
        sessionsList.querySelectorAll('.session-item').forEach(item => {
            item.addEventListener('click', () => {
                this.selectSession(item.dataset.sessionId);
            });
        });
    }

    selectSession(sessionId) {
        this.currentSession = sessionId;
        
        // Update UI
        document.querySelectorAll('.session-item').forEach(item => {
            item.classList.toggle('active', item.dataset.sessionId === sessionId);
        });

        // Enable debug controls
        this.updateDebugControls(true);
        document.getElementById('btn-add-breakpoint').disabled = false;

        // Load session data
        this.refreshCurrentSessionData();
        
        this.addLogEntry('info', `Selected session: ${sessionId}`);
    }

    updateDebugControls(enabled) {
        document.getElementById('btn-continue').disabled = !enabled;
        document.getElementById('btn-step').disabled = !enabled;
        document.getElementById('btn-stop').disabled = !enabled;
    }

    executeDebugCommand(command) {
        if (!this.currentSession) {
            this.addLogEntry('error', 'No session selected');
            return;
        }

        const commandData = {
            id: Date.now(),
            type: command,
            sessionId: this.currentSession
        };

        this.socket.emit('debug_command', commandData);
        this.addLogEntry('debug', `Executing command: ${command}`);
    }

    refreshSessions() {
        this.socket.emit('get_sessions');
    }

    refreshCurrentSessionData() {
        if (!this.currentSession) return;

        if (this.autoRefreshVars) {
            this.socket.emit('get_variables', { sessionId: this.currentSession });
        }

        if (this.autoRefreshRegs) {
            this.socket.emit('get_registers', { sessionId: this.currentSession });
        }
    }

    updateVariables(variables) {
        const content = document.getElementById('variables-content');
        
        if (!variables || variables.length === 0) {
            content.innerHTML = '<div class="no-data">No variables available</div>';
            return;
        }

        content.innerHTML = variables.map(variable => `
            <div class="variable-item">
                <div>
                    <span class="variable-name">${variable.name}</span>
                    <span class="variable-type">(${variable.type || 'unknown'})</span>
                </div>
                <span class="variable-value">${variable.value || 'N/A'}</span>
            </div>
        `).join('');
    }

    updateRegisters(registers) {
        const content = document.getElementById('registers-content');
        
        if (!registers || registers.length === 0) {
            content.innerHTML = '<div class="no-data">No registers available</div>';
            return;
        }

        content.innerHTML = registers.map(register => `
            <div class="register-item">
                <span class="register-name">${register.name}</span>
                <span class="register-value">0x${register.value || '00000000'}</span>
            </div>
        `).join('');
    }

    addLogEntry(level, message, timestamp = null) {
        const logsContent = document.getElementById('logs-content');
        const time = timestamp || new Date().toISOString();
        
        const logEntry = document.createElement('div');
        logEntry.className = `log-entry ${level}`;
        logEntry.innerHTML = `
            <span class="timestamp">[${time.split('T')[1].split('.')[0]}]</span>
            <span class="message">${message}</span>
        `;
        
        logsContent.appendChild(logEntry);
        
        // Auto-scroll if enabled
        if (this.autoScrollLogs) {
            logsContent.scrollTop = logsContent.scrollHeight;
        }
        
        // Limit log entries
        const entries = logsContent.querySelectorAll('.log-entry');
        if (entries.length > 100) {
            entries[0].remove();
        }
    }

    showModal(modalId) {
        document.getElementById(modalId).classList.add('show');
    }

    hideModal(modalId) {
        document.getElementById(modalId).classList.remove('show');
    }

    createNewSession() {
        const form = document.getElementById('new-session-form');
        const formData = new FormData(form);
        
        const sessionData = {
            program: formData.get('program'),
            gdb_path: formData.get('gdb_path'),
            name: formData.get('name') || 'Debug Session'
        };

        // Send to server via API
        fetch('/api/sessions', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(sessionData)
        })
        .then(response => response.json())
        .then(session => {
            this.addLogEntry('info', `Session created: ${session.name || session.id}`);
            this.hideModal('new-session-modal');
            form.reset();
            this.refreshSessions();
        })
        .catch(error => {
            this.addLogEntry('error', `Failed to create session: ${error.message}`);
        });
    }

    addBreakpoint() {
        if (!this.currentSession) {
            this.addLogEntry('error', 'No session selected');
            return;
        }

        const form = document.getElementById('add-breakpoint-form');
        const formData = new FormData(form);
        
        const breakpointData = {
            sessionId: this.currentSession,
            file: formData.get('file'),
            line: parseInt(formData.get('line')),
            condition: formData.get('condition') || null
        };

        this.socket.emit('set_breakpoint', breakpointData);
        this.hideModal('add-breakpoint-modal');
        form.reset();
    }
}

    // Auto-refresh functionality
    startAutoRefresh() {
        setInterval(() => {
            if (this.currentSession && this.socket.connected) {
                this.refreshCurrentSessionData();
            }
        }, 2000); // Refresh every 2 seconds
    }

    // Utility methods
    formatTimestamp(timestamp) {
        return new Date(timestamp).toLocaleTimeString();
    }

    formatValue(value, type) {
        if (type === 'pointer' || type === 'address') {
            return `0x${value.toString(16).padStart(8, '0')}`;
        }
        return value;
    }

    // Keyboard shortcuts
    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            if (e.ctrlKey || e.metaKey) {
                switch (e.key) {
                    case 'Enter':
                        e.preventDefault();
                        if (this.currentSession) {
                            this.executeDebugCommand('continue');
                        }
                        break;
                    case 'F10':
                        e.preventDefault();
                        if (this.currentSession) {
                            this.executeDebugCommand('step');
                        }
                        break;
                    case 'F5':
                        e.preventDefault();
                        this.refreshCurrentSessionData();
                        break;
                }
            }
        });
    }

    // Initialize everything
    init() {
        this.startAutoRefresh();
        this.setupKeyboardShortcuts();
        this.addLogEntry('info', 'Debug dashboard initialized');
    }
}

// Initialize dashboard when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    const dashboard = new DebugDashboard();
    dashboard.init();
});
