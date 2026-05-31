import { invoke, fs, openDialog } from './services/tauri.js';
import { showStatus } from './utils.js';

/**
 * Convert Uint8Array to Base64 safely without stack overflow for large files
 * @param {Uint8Array} uint8Array 
 * @returns {string} Base64 encoded string
 */
function uint8ArrayToBase64(uint8Array) {
    let binary = '';
    const len = uint8Array.byteLength;
    for (let i = 0; i < len; i++) {
        binary += String.fromCharCode(uint8Array[i]);
    }
    return window.btoa(binary);
}

/**
 * Apply selected theme colors and Shoelace built-in themes
 * @param {string} themeName - 'dark', 'light', or 'yellow'
 */
function applyTheme(themeName) {
    // Remove existing custom theme classes
    document.body.classList.remove('theme-dark', 'theme-light', 'theme-yellow');
    // Add new theme class
    document.body.classList.add(`theme-${themeName}`);

    // Synchronize Shoelace component themes
    if (themeName === 'dark') {
        document.body.classList.remove('sl-theme-light');
        document.body.classList.add('sl-theme-dark');
        document.documentElement.classList.remove('sl-theme-light');
        document.documentElement.classList.add('sl-theme-dark');
    } else {
        // Both Light and Yellow themes will use Shoelace's light theme base
        document.body.classList.remove('sl-theme-dark');
        document.body.classList.add('sl-theme-light');
        document.documentElement.classList.remove('sl-theme-dark');
        document.documentElement.classList.add('sl-theme-light');
    }
}

/**
 * Apply background image with opacity and blur styles
 * @param {string} base64Image - Base64 Data URL of the image
 * @param {number} opacity - 0 to 100 value
 * @param {number} blur - Blur value in pixels
 */
function applyBackground(base64Image, opacity, blur) {
    const bgElement = document.getElementById('app-background');
    const opacityGroup = document.getElementById('bg-opacity-group');
    const blurGroup = document.getElementById('bg-blur-group');

    if (!bgElement) return;

    if (base64Image) {
        document.body.classList.add('has-background');
        bgElement.style.backgroundImage = `url("${base64Image}")`;
        document.documentElement.style.setProperty('--bg-opacity', opacity / 100);
        document.documentElement.style.setProperty('--bg-blur', `${blur}px`);

        if (opacityGroup) opacityGroup.style.display = 'block';
        if (blurGroup) blurGroup.style.display = 'block';
    } else {
        document.body.classList.remove('has-background');
        bgElement.style.backgroundImage = 'none';
        
        if (opacityGroup) opacityGroup.style.display = 'none';
        if (blurGroup) blurGroup.style.display = 'none';
    }

    // Force browser repaint on the sidebar to fix backdrop-filter layout bug in WebKit
    const sidebar = document.getElementById('sidebar');
    if (sidebar) {
        const originalDisplay = sidebar.style.display;
        sidebar.style.display = 'none';
        sidebar.offsetHeight; // Read to trigger reflow
        sidebar.style.display = originalDisplay;
    }
}

export async function initSettings() {
    // 1. Initial Core Elements
    const apiKeyInput = document.getElementById('settings-api-key');
    const modelSelect = document.getElementById('settings-model');
    const saveSettingsBtn = document.getElementById('save-settings-btn');

    // 2. Theme & Background Elements
    const themeSelect = document.getElementById('settings-theme');
    const selectBgBtn = document.getElementById('select-bg-btn');
    const clearBgBtn = document.getElementById('clear-bg-btn');
    const opacitySlider = document.getElementById('settings-bg-opacity');
    const opacityValText = document.getElementById('bg-opacity-val');
    const blurSlider = document.getElementById('settings-bg-blur');
    const blurValText = document.getElementById('bg-blur-val');

    // 3. Load & Apply Saved UI Settings from localStorage
    const savedTheme = localStorage.getItem('ui_theme') || 'dark';
    const savedBgImage = localStorage.getItem('ui_bg_image') || '';
    const savedBgOpacity = parseInt(localStorage.getItem('ui_bg_opacity') || '50', 10);
    const savedBgBlur = parseInt(localStorage.getItem('ui_bg_blur') || '0', 10);

    // Apply UI parameters immediately
    applyTheme(savedTheme);
    applyBackground(savedBgImage, savedBgOpacity, savedBgBlur);

    // Populate saved UI values to elements if they exist
    if (themeSelect) themeSelect.value = savedTheme;
    if (opacitySlider) {
        opacitySlider.value = savedBgOpacity;
        if (opacityValText) opacityValText.innerText = `${savedBgOpacity}%`;
    }
    if (blurSlider) {
        blurSlider.value = savedBgBlur;
        if (blurValText) blurValText.innerText = `${savedBgBlur}px`;
    }

    // 4. Load API settings from Backend
    if (apiKeyInput && modelSelect) {
        try {
            const settings = await invoke('get_settings');
            apiKeyInput.value = settings.api_key;
            if (settings.model) {
                modelSelect.value = settings.model;
                const aiModelStatus = document.getElementById('ai-model');
                if (aiModelStatus) aiModelStatus.innerText = settings.model;
            }
        } catch (err) {
            console.error("Failed to load backend settings:", err);
        }
    }

    // 5. Setup Theme Selection Listeners
    if (themeSelect) {
        themeSelect.addEventListener('sl-change', () => {
            const selectedTheme = themeSelect.value;
            applyTheme(selectedTheme);
            localStorage.setItem('ui_theme', selectedTheme);
            showStatus(`Đã đổi sang giao diện ${selectedTheme === 'dark' ? 'Tối' : selectedTheme === 'light' ? 'Sáng' : 'Vàng dịu mắt'}`);
        });
    }

    // 6. Setup Background Image Selection Listener
    if (selectBgBtn) {
        selectBgBtn.onclick = async () => {
            try {
                const selected = await openDialog({
                    directory: false,
                    multiple: false,
                    title: "Chọn ảnh nền cho ứng dụng",
                    filters: [{
                        name: 'Images',
                        extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif']
                    }]
                });

                if (selected) {
                    showStatus("Đang nạp hình nền...");
                    
                    // Read file as binary via Tauri FS Plugin
                    const fileData = await fs.readFile(selected);
                    const base64Str = uint8ArrayToBase64(fileData);
                    
                    // Detect content type based on extension
                    const ext = selected.split('.').pop().toLowerCase();
                    const mimeType = ['png', 'webp', 'gif'].includes(ext) ? `image/${ext}` : 'image/jpeg';
                    const dataUrl = `data:${mimeType};base64,${base64Str}`;

                    // Update UI state
                    const currentOpacity = opacitySlider ? opacitySlider.value : 50;
                    const currentBlur = blurSlider ? blurSlider.value : 0;

                    applyBackground(dataUrl, currentOpacity, currentBlur);
                    
                    // Save to local storage
                    localStorage.setItem('ui_bg_image', dataUrl);
                    showStatus("Đã thiết lập hình nền thành công!");
                }
            } catch (err) {
                console.error("Failed to select or process background image:", err);
                showStatus("Lỗi tải hình nền!", true);
            }
        };
    }

    // 7. Setup Background Clear Listener
    if (clearBgBtn) {
        clearBgBtn.onclick = () => {
            applyBackground('', 50, 0);
            localStorage.setItem('ui_bg_image', '');
            showStatus("Đã xóa hình nền, khôi phục nền trơn.");
        };
    }

    // 8. Setup Opacity Slider Listener
    if (opacitySlider) {
        opacitySlider.addEventListener('sl-input', () => {
            const val = opacitySlider.value;
            if (opacityValText) opacityValText.innerText = `${val}%`;
            document.documentElement.style.setProperty('--bg-opacity', val / 100);
            localStorage.setItem('ui_bg_opacity', val);
        });
    }

    // 9. Setup Blur Slider Listener
    if (blurSlider) {
        blurSlider.addEventListener('sl-input', () => {
            const val = blurSlider.value;
            if (blurValText) blurValText.innerText = `${val}px`;
            document.documentElement.style.setProperty('--bg-blur', `${val}px`);
            localStorage.setItem('ui_bg_blur', val);
        });
    }

    // 10. Core Save Settings (API & Model config)
    if (saveSettingsBtn && apiKeyInput && modelSelect) {
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
                
                const aiModelStatus = document.getElementById('ai-model');
                if (aiModelStatus) aiModelStatus.innerText = model;

            } catch (err) {
                console.error("Failed to save settings:", err);
                showStatus("Lỗi lưu cấu hình!", true);
            } finally {
                saveSettingsBtn.disabled = false;
                saveSettingsBtn.innerText = "Lưu Cấu Hình";
            }
        };
    }
}
