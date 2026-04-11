import { invoke } from './services/tauri.js';
import { showStatus } from './utils.js';

export async function initSettings() {
    const apiKeyInput = document.getElementById('settings-api-key');
    const modelSelect = document.getElementById('settings-model');
    const saveSettingsBtn = document.getElementById('save-settings-btn');

    if (!apiKeyInput || !modelSelect || !saveSettingsBtn) return;

    // Load initial settings
    try {
        const settings = await invoke('get_settings');
        apiKeyInput.value = settings.api_key;
        // Ensure the correct model is selected and status bar is updated
        if (settings.model) {
            modelSelect.value = settings.model;
            // For Shoelace sl-select, sometimes we need to set the value explicitly after a short delay
            // if it's still being updated, but usually direct assignment works if options exist.
            const aiModelStatus = document.getElementById('ai-model');
            if (aiModelStatus) aiModelStatus.innerText = settings.model;
        }
    } catch (err) {
        console.error("Failed to load settings:", err);
    }

    saveSettingsBtn.onclick = async () => {
        const apiKey = apiKeyInput.value.trim();
        const model = modelSelect.value;

        if (!apiKey) {
            showStatus("API Key không được để trống!");
            return;
        }

        try {
            saveSettingsBtn.disabled = true;
            saveSettingsBtn.innerText = "Đang lưu...";
            
            await invoke('save_settings', { apiKey, model });
            
            showStatus("Đã lưu cấu hình thành công!");
            
            // Update UI status bar if exists
            const aiModelStatus = document.getElementById('ai-model');
            if (aiModelStatus) aiModelStatus.innerText = model;

        } catch (err) {
            console.error("Failed to save settings:", err);
            showStatus("Lỗi khi lưu cấu hình: " + err);
        } finally {
            saveSettingsBtn.disabled = false;
            saveSettingsBtn.innerText = "Lưu Cấu Hình";
        }
    };
}

