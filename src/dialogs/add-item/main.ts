import { invokeCommand, listenEvent } from '../../shared/ipc';
import { ICONS, getIconSvg, getCategoryDefaultIcon, getSystemIcon, type IconName } from '../../shared/icons';

let currentKind: 'app' | 'terminal' | 'folder' | 'system' = 'app';
let appRunAsAdmin = false;
let termNotifyOnDone = false;
let currentIcon: string | null = null;

// DOM Elements
const kindTabs = document.querySelectorAll<HTMLButtonElement>('.kind-tab');
const sections = {
  app: document.getElementById('section-app')!,
  terminal: document.getElementById('section-terminal')!,
  folder: document.getElementById('section-folder')!,
  system: document.getElementById('section-system')!
};

const inpLabel = document.getElementById('inp-label') as HTMLInputElement;
const selCategory = document.getElementById('sel-category') as HTMLSelectElement;
const previewText = document.getElementById('preview-text') as unknown as SVGTextElement;
const previewIconWrap = document.getElementById('preview-icon-wrap') as SVGGElement | null;
const inpAppPath = document.getElementById('inp-app-path') as HTMLInputElement;
const inpAppArgs = document.getElementById('inp-app-args') as HTMLInputElement;
const selTermAction = document.getElementById('sel-term-action') as HTMLSelectElement;
const selTermShell = document.getElementById('sel-term-shell') as HTMLSelectElement;
const inpTermCmd = document.getElementById('inp-term-cmd') as HTMLInputElement;
const inpFolderTarget = document.getElementById('inp-folder-target') as HTMLInputElement;
const selSystemFn = document.getElementById('sel-system-fn') as HTMLSelectElement;
const togAppAdmin = document.getElementById('tog-app-admin');
const togTermNotify = document.getElementById('tog-term-notify');
const togAutostart = document.getElementById('tog-autostart');
const autostartStatusText = document.getElementById('autostart-status-text');
let autostartEnabled = false;

// Icon Elements
const iconPreviewBox = document.getElementById('icon-preview-box') as HTMLDivElement | null;
const iconDropZone = document.getElementById('icon-drop-zone') as HTMLDivElement | null;
const inpIconFile = document.getElementById('inp-icon-file') as HTMLInputElement | null;
const btnBrowseIcon = document.getElementById('btn-browse-icon') as HTMLButtonElement | null;
const selPresetIcon = document.getElementById('sel-preset-icon') as HTMLSelectElement | null;
const btnClearIcon = document.getElementById('btn-clear-icon') as HTMLButtonElement | null;

function getDefaultIconForCurrentState(): IconName {
  if (currentKind === 'system') {
    return getSystemIcon(selSystemFn?.value || 'lock');
  }
  return getCategoryDefaultIcon(currentKind);
}

function updateIconUI() {
  const isImage = currentIcon ? (
    currentIcon.startsWith('data:image/') ||
    currentIcon.startsWith('http://') ||
    currentIcon.startsWith('https://') ||
    /\.(png|jpe?g|svg|ico|webp)$/i.test(currentIcon)
  ) : false;

  // 1. Drop zone text
  if (iconDropZone) {
    if (isImage) {
      iconDropZone.textContent = '커스텀 이미지 적용됨 (클릭하여 다른 이미지 선택)';
    } else if (currentIcon) {
      iconDropZone.textContent = `프리셋 '${currentIcon}' 적용됨 (클릭하여 이미지 선택)`;
    } else {
      iconDropZone.textContent = '이미지 파일을 여기에 드래그하거나 클릭하여 파일 탐색기 열기';
    }
  }

  // 2. Preset dropdown sync
  if (selPresetIcon) {
    if (currentIcon && !isImage) {
      selPresetIcon.value = currentIcon;
    } else {
      selPresetIcon.value = '';
    }
  }

  // 3. icon-preview-box (40x40)
  if (iconPreviewBox) {
    iconPreviewBox.innerHTML = '';
    if (isImage && currentIcon) {
      const img = document.createElement('img');
      img.src = currentIcon;
      img.alt = '선택된 아이콘';
      iconPreviewBox.appendChild(img);
    } else if (currentIcon) {
      iconPreviewBox.innerHTML = getIconSvg(currentIcon, 24);
    } else {
      const def = getDefaultIconForCurrentState();
      iconPreviewBox.innerHTML = getIconSvg(def, 24, { stroke: 'rgba(143,233,245,0.45)' });
    }
  }

  // 4. Sidebar circular preview-box
  if (previewIconWrap) {
    previewIconWrap.innerHTML = '';
    if (isImage && currentIcon) {
      const imgEl = document.createElementNS('http://www.w3.org/2000/svg', 'image');
      imgEl.setAttribute('href', currentIcon);
      imgEl.setAttribute('x', '0');
      imgEl.setAttribute('y', '0');
      imgEl.setAttribute('width', '24');
      imgEl.setAttribute('height', '24');
      imgEl.setAttribute('preserveAspectRatio', 'xMidYMid meet');
      previewIconWrap.appendChild(imgEl);
      if (previewText) previewText.style.display = 'none';
    } else if (currentIcon) {
      const iconDef = (ICONS as Record<string, string>)[currentIcon] || ICONS.window;
      const svgEl = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svgEl.setAttribute('width', '24');
      svgEl.setAttribute('height', '24');
      svgEl.setAttribute('viewBox', '0 0 24 24');
      svgEl.setAttribute('fill', 'none');
      svgEl.setAttribute('stroke', '#8FE9F5');
      svgEl.style.color = '#8FE9F5';
      svgEl.setAttribute('stroke-width', '1.5');
      svgEl.setAttribute('stroke-linecap', 'round');
      svgEl.setAttribute('stroke-linejoin', 'round');
      svgEl.innerHTML = iconDef;
      previewIconWrap.appendChild(svgEl);
      if (previewText) previewText.style.display = 'none';
    } else {
      if (previewText) {
        previewText.style.display = '';
        const txt = inpLabel.value.trim();
        previewText.textContent = txt ? txt.substring(0, 4) : 'NEW';
      }
    }
  }
}

function processImageDataUrl(dataUrl: string): Promise<string> {
  return new Promise((resolve) => {
    if (dataUrl.startsWith('data:image/svg+xml')) {
      resolve(dataUrl);
      return;
    }

    const img = new Image();
    img.onload = () => {
      const maxDim = 128;
      if (img.width <= maxDim && img.height <= maxDim) {
        resolve(dataUrl);
        return;
      }
      const canvas = document.createElement('canvas');
      let w = img.width;
      let h = img.height;
      if (w > h) {
        h = Math.round((h * maxDim) / w);
        w = maxDim;
      } else {
        w = Math.round((w * maxDim) / h);
        h = maxDim;
      }
      canvas.width = Math.max(1, w);
      canvas.height = Math.max(1, h);
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = 'high';
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
        resolve(canvas.toDataURL('image/png'));
      } else {
        resolve(dataUrl);
      }
    };
    img.onerror = () => resolve(dataUrl);
    img.src = dataUrl;
  });
}

async function handleImageFile(file: File) {
  if (!file.type.startsWith('image/') && !/\.(png|jpe?g|svg|ico|webp)$/i.test(file.name)) {
    alert('지원되는 이미지 파일(PNG, JPG, SVG, ICO, WEBP)을 선택해주세요.');
    return;
  }
  const reader = new FileReader();
  reader.onload = async () => {
    const rawDataUrl = reader.result as string;
    const processed = await processImageDataUrl(rawDataUrl);
    currentIcon = processed;
    updateIconUI();
  };
  reader.readAsDataURL(file);
}

function handleImageUrl(url: string) {
  const trimmed = url.trim();
  if (
    trimmed.startsWith('data:image/') ||
    trimmed.startsWith('http://') ||
    trimmed.startsWith('https://')
  ) {
    currentIcon = trimmed;
    updateIconUI();
  } else {
    alert('유효한 이미지 URL이나 data URI 형식이 아닙니다.');
  }
}

function resetForm() {
  inpLabel.value = '';
  previewText.textContent = 'NEW';
  inpAppPath.value = '';
  inpAppArgs.value = '';
  inpTermCmd.value = '';
  inpFolderTarget.value = '';
  appRunAsAdmin = false;
  togAppAdmin?.classList.remove('on');
  termNotifyOnDone = false;
  togTermNotify?.classList.remove('on');
  currentIcon = null;
  if (selPresetIcon) selPresetIcon.value = '';
  if (inpIconFile) inpIconFile.value = '';
  updateIconUI();
}

let currentConfig: any = null;
const itemsListContainer = document.getElementById('items-list') as HTMLDivElement;

function renderItemsList() {
  if (!itemsListContainer) return;
  itemsListContainer.innerHTML = '';

  const catId = selCategory.value;
  const currentCat = currentConfig?.categories?.find((c: any) =>
    c.id === catId || c.id.startsWith(catId) || catId.startsWith(c.id)
  );

  if (!currentCat || !currentCat.items || currentCat.items.length === 0) {
    const emptyNotice = document.createElement('span');
    emptyNotice.style.fontSize = '11.5px';
    emptyNotice.style.color = 'var(--rm-color-text-secondary, #9FB4B6)';
    emptyNotice.style.fontStyle = 'italic';
    emptyNotice.textContent = '등록된 항목 없음';
    itemsListContainer.appendChild(emptyNotice);
    return;
  }

  for (const item of currentCat.items) {
    const chip = document.createElement('div');
    chip.className = 'item-chip';

    const lbl = document.createElement('span');
    lbl.style.overflow = 'hidden';
    lbl.style.textOverflow = 'ellipsis';
    lbl.style.whiteSpace = 'nowrap';
    lbl.style.maxWidth = '120px';
    lbl.textContent = item.label || item.id;
    lbl.title = item.label || item.id;

    const delBtn = document.createElement('button');
    delBtn.className = 'btn-del-item';
    delBtn.textContent = '삭제';
    delBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      if (confirm(`'${item.label || item.id}' 항목을 삭제하시겠습니까?`)) {
        try {
          await invokeCommand('remove_menu_item', { itemId: item.id });
          await loadCategories(selCategory.value);
        } catch (err) {
          alert('항목 삭제 실패: ' + err);
        }
      }
    });

    chip.appendChild(lbl);
    chip.appendChild(delBtn);
    itemsListContainer.appendChild(chip);
  }
}

async function loadCategories(preselectId?: string) {
  try {
    const config = await invokeCommand<any>('get_menu_config');
    currentConfig = config;
    if (config && config.categories && config.categories.length > 0) {
      selCategory.innerHTML = '';
      for (const cat of config.categories) {
        const opt = document.createElement('option');
        opt.value = cat.id;
        opt.textContent = `${cat.label} (${cat.kind})`;
        opt.setAttribute('data-kind', cat.kind);
        selCategory.appendChild(opt);
      }
    }
  } catch (err) {
    console.error('Failed to load categories:', err);
  }

  if (preselectId) {
    const matchingOption = Array.from(selCategory.options).find(
      o => o.value === preselectId || o.value.startsWith(preselectId) || preselectId.startsWith(o.value)
    );
    if (matchingOption) {
      selCategory.value = matchingOption.value;
    }
  }
  syncTabWithCategory();
  renderItemsList();
}

function syncTabWithCategory() {
  const selectedOpt = selCategory.selectedOptions[0];
  const kind = selectedOpt?.getAttribute('data-kind') || 
    (selCategory.value.startsWith('app') ? 'app' :
     selCategory.value.startsWith('term') ? 'terminal' :
     selCategory.value.startsWith('fold') ? 'folder' : 'system');
  const matchingTab = Array.from(kindTabs).find(t => t.getAttribute('data-kind') === kind);
  if (matchingTab && !matchingTab.classList.contains('active')) {
    matchingTab.click();
  }
  renderItemsList();
}

// Initial category loading & autostart status
loadCategories();
fetchAutostartStatus();
updateIconUI();

// Listen to dialog initialization from radial menu
listenEvent<{ categoryId?: string }>('dialog:init', (payload) => {
  resetForm();
  loadCategories(payload?.categoryId);
  fetchAutostartStatus();
  updateIconUI();
});

// Tab Switcher
kindTabs.forEach(tab => {
  tab.addEventListener('click', () => {
    kindTabs.forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    currentKind = tab.getAttribute('data-kind') as any;

    Object.entries(sections).forEach(([k, el]) => {
      el.style.display = k === currentKind ? 'block' : 'none';
    });

    const matchingOption = Array.from(selCategory.options).find(
      o => o.getAttribute('data-kind') === currentKind || o.value.startsWith(currentKind) || (currentKind === 'folder' && o.value.startsWith('fold'))
    );
    if (matchingOption && selCategory.value !== matchingOption.value) {
      selCategory.value = matchingOption.value;
    }
    updateIconUI();
  });
});

// Category Switcher
selCategory.addEventListener('change', () => {
  syncTabWithCategory();
});

// App Admin Toggle
togAppAdmin?.addEventListener('click', () => {
  appRunAsAdmin = !appRunAsAdmin;
  togAppAdmin.classList.toggle('on', appRunAsAdmin);
});

// Terminal Notify Toggle
togTermNotify?.addEventListener('click', () => {
  termNotifyOnDone = !termNotifyOnDone;
  togTermNotify.classList.toggle('on', termNotifyOnDone);
});

// Autostart Functions & Listeners
function updateAutostartUI(enabled: boolean) {
  autostartEnabled = enabled;
  if (togAutostart) {
    togAutostart.classList.toggle('on', enabled);
  }
  if (autostartStatusText) {
    autostartStatusText.textContent = enabled ? '현재 켜짐 (작업 스케줄러 등록)' : '현재 꺼짐';
    autostartStatusText.style.color = enabled ? '#8FE9F5' : 'var(--rm-color-text-secondary, #9FB4B6)';
  }
}

async function fetchAutostartStatus() {
  try {
    const enabled = await invokeCommand<boolean>('get_autostart_status');
    updateAutostartUI(enabled);
  } catch (err) {
    console.error('Failed to get autostart status:', err);
    if (autostartStatusText) {
      autostartStatusText.textContent = '상태 확인 실패';
    }
  }
}

togAutostart?.addEventListener('click', async () => {
  const next = !autostartEnabled;
  try {
    const updated = await invokeCommand<boolean>('set_autostart_status', { enabled: next });
    updateAutostartUI(updated);
  } catch (err) {
    console.error('Failed to update autostart status:', err);
    alert('자동 시작 상태 변경 실패: ' + err);
    fetchAutostartStatus();
  }
});

listenEvent<{ enabled: boolean }>('autostart:changed', (payload) => {
  if (payload && typeof payload.enabled === 'boolean') {
    updateAutostartUI(payload.enabled);
  }
});

// Browse App Path
document.getElementById('btn-browse-app')?.addEventListener('click', async () => {
  try {
    const filePath = await invokeCommand<string | null>('browse_file');
    if (filePath) {
      inpAppPath.value = filePath;
      if (!inpLabel.value.trim()) {
        const fileName = filePath.split(/[\\/]/).pop() || '';
        const baseName = fileName.replace(/\.[^/.]+$/, '');
        inpLabel.value = baseName;
        previewText.textContent = baseName.substring(0, 4);
        updateIconUI();
      }
    }
  } catch (err) {
    console.error('File browse error:', err);
  }
});

// Browse Folder Target
document.getElementById('btn-browse-folder')?.addEventListener('click', async () => {
  try {
    const folderPath = await invokeCommand<string | null>('browse_folder');
    if (folderPath) {
      inpFolderTarget.value = folderPath;
      if (!inpLabel.value.trim()) {
        const folderName = folderPath.split(/[\\/]/).filter(Boolean).pop() || '';
        inpLabel.value = folderName;
        previewText.textContent = folderName.substring(0, 4);
        updateIconUI();
      }
    }
  } catch (err) {
    console.error('Folder browse error:', err);
  }
});

// Placeholder Chips
document.querySelectorAll<HTMLSpanElement>('.chip').forEach(chip => {
  chip.addEventListener('click', () => {
    const val = chip.getAttribute('data-val') || '';
    if (inpTermCmd) {
      const start = inpTermCmd.selectionStart || inpTermCmd.value.length;
      const end = inpTermCmd.selectionEnd || inpTermCmd.value.length;
      inpTermCmd.value = inpTermCmd.value.substring(0, start) + val + inpTermCmd.value.substring(end);
      inpTermCmd.focus();
    }
  });
});

// Label Input -> Preview Update
inpLabel.addEventListener('input', () => {
  const text = inpLabel.value.trim();
  previewText.textContent = text ? text.substring(0, 4) : 'NEW';
  updateIconUI();
});

// Icon Drag & Drop, File Input, Preset & Clipboard Events
if (iconDropZone) {
  iconDropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    e.stopPropagation();
    iconDropZone.classList.add('drag-over');
  });

  iconDropZone.addEventListener('dragleave', (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (!iconDropZone.contains(e.relatedTarget as Node)) {
      iconDropZone.classList.remove('drag-over');
    }
  });

  iconDropZone.addEventListener('drop', (e) => {
    e.preventDefault();
    e.stopPropagation();
    iconDropZone.classList.remove('drag-over');

    const dt = e.dataTransfer;
    if (!dt) return;

    if (dt.files && dt.files.length > 0) {
      handleImageFile(dt.files[0]);
    } else {
      const uri = dt.getData('text/uri-list') || dt.getData('text/plain');
      if (uri) {
        handleImageUrl(uri);
      }
    }
  });

  iconDropZone.addEventListener('click', () => {
    inpIconFile?.click();
  });
}

inpIconFile?.addEventListener('change', () => {
  if (inpIconFile.files && inpIconFile.files.length > 0) {
    handleImageFile(inpIconFile.files[0]);
    inpIconFile.value = '';
  }
});

btnBrowseIcon?.addEventListener('click', () => {
  inpIconFile?.click();
});

selPresetIcon?.addEventListener('change', () => {
  const val = selPresetIcon.value;
  currentIcon = val || null;
  updateIconUI();
});

btnClearIcon?.addEventListener('click', () => {
  currentIcon = null;
  if (selPresetIcon) selPresetIcon.value = '';
  if (inpIconFile) inpIconFile.value = '';
  updateIconUI();
});

window.addEventListener('paste', (e) => {
  const items = e.clipboardData?.items;
  if (items) {
    for (let i = 0; i < items.length; i++) {
      if (items[i].type.startsWith('image/')) {
        const file = items[i].getAsFile();
        if (file) {
          handleImageFile(file);
          return;
        }
      }
    }
  }

  const text = e.clipboardData?.getData('text');
  if (
    text &&
    (text.startsWith('data:image/') ||
      ((text.startsWith('http://') || text.startsWith('https://')) &&
        /\.(png|jpe?g|svg|ico|webp)$/i.test(text)))
  ) {
    handleImageUrl(text);
  }
});

selSystemFn?.addEventListener('change', () => {
  updateIconUI();
});

// Close / Cancel
const closeDialog = async () => {
  try {
    await invokeCommand('close_dialog_window', { dialogLabel: 'add-item' });
  } catch {
    window.close();
  }
};
document.getElementById('btn-close')?.addEventListener('click', closeDialog);
document.getElementById('btn-cancel')?.addEventListener('click', closeDialog);

// Save Action
document.getElementById('btn-save')?.addEventListener('click', async () => {
  const label = inpLabel.value.trim();
  if (!label) {
    alert('항목 이름을 입력하세요.');
    inpLabel.focus();
    return;
  }

  let itemData: any = {
    label,
    kind: currentKind,
    icon: currentIcon || undefined
  };

  if (currentKind === 'app') {
    const path = inpAppPath.value.trim();
    if (!path) {
      alert('실행 파일 경로를 입력하세요.');
      return;
    }
    const args = inpAppArgs.value.trim();
    itemData = {
      ...itemData,
      path,
      args: args || undefined,
      whenRunning: 'focus',
      runAsAdmin: appRunAsAdmin
    };
  } else if (currentKind === 'terminal') {
    const action = selTermAction.value;
    const shell = selTermShell.value;
    const command = inpTermCmd.value.trim();
    if (action === 'run' && !command) {
      alert('실행할 명령어를 입력하세요.');
      return;
    }
    itemData = {
      ...itemData,
      action,
      shell,
      mode: 'command',
      command: action === 'run' ? command : undefined,
      startDir: action === 'open' ? '{현재 폴더}' : undefined,
      notifyOnDone: termNotifyOnDone,
      runAsAdmin: false
    };
  } else if (currentKind === 'folder') {
    const target = inpFolderTarget.value.trim();
    if (!target) {
      alert('대상 폴더 또는 URL을 입력하세요.');
      return;
    }
    itemData = {
      ...itemData,
      target,
      openIn: 'explorer'
    };
  } else if (currentKind === 'system') {
    const fn = selSystemFn.value;
    itemData = {
      ...itemData,
      fn
    };
  }

  try {
    await invokeCommand('add_menu_item', {
      categoryId: selCategory.value,
      item: itemData
    });
    alert('항목이 성공적으로 추가되었습니다!');
    closeDialog();
  } catch (err) {
    console.error('Save failed:', err);
    alert('항목 추가 중 오류가 발생했습니다: ' + err);
  }
});
