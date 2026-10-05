import { invokeCommand, listenEvent } from '../../shared/ipc';

let currentKind: 'app' | 'terminal' | 'folder' | 'system' = 'app';
let appRunAsAdmin = false;
let termNotifyOnDone = false;

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
const inpAppPath = document.getElementById('inp-app-path') as HTMLInputElement;
const inpAppArgs = document.getElementById('inp-app-args') as HTMLInputElement;
const selTermAction = document.getElementById('sel-term-action') as HTMLSelectElement;
const selTermShell = document.getElementById('sel-term-shell') as HTMLSelectElement;
const inpTermCmd = document.getElementById('inp-term-cmd') as HTMLInputElement;
const inpFolderTarget = document.getElementById('inp-folder-target') as HTMLInputElement;
const selSystemFn = document.getElementById('sel-system-fn') as HTMLSelectElement;
const togAppAdmin = document.getElementById('tog-app-admin');
const togTermNotify = document.getElementById('tog-term-notify');

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

// Initial category loading
loadCategories();

// Listen to dialog initialization from radial menu
listenEvent<{ categoryId?: string }>('dialog:init', (payload) => {
  resetForm();
  loadCategories(payload?.categoryId);
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
    kind: currentKind
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
