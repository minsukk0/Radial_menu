import { invokeCommand } from '../../shared/ipc';

// 0.5s Anti-click delay enforcement
const btnApprove = document.getElementById('btn-approve') as HTMLButtonElement;
const btnOnce = document.getElementById('btn-once') as HTMLButtonElement;
const btnCancel = document.getElementById('btn-cancel') as HTMLButtonElement;
const countdownNote = document.getElementById('countdown-note') as HTMLSpanElement;

const valLabel = document.getElementById('val-label') as HTMLSpanElement;
const valTarget = document.getElementById('val-target') as HTMLSpanElement;
const valHash = document.getElementById('val-hash') as HTMLSpanElement;

// Read query params if passed, or default mock
const params = new URLSearchParams(window.location.search);
const itemId = params.get('itemId') || 'item-unknown';
valLabel.textContent = params.get('label') || '관리자 터미널 명령';
valTarget.textContent = params.get('target') || 'powershell.exe -ExecutionPolicy Bypass';
valHash.textContent = params.get('hash') || 'sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';

// Enable buttons after 500ms
setTimeout(() => {
  btnApprove.disabled = false;
  btnOnce.disabled = false;
  if (countdownNote) {
    countdownNote.textContent = '';
  }
}, 500);

const closeDialog = async () => {
  try {
    await invokeCommand('close_dialog_window', { dialogLabel: 'approve' });
  } catch {
    window.close();
  }
};

btnCancel.addEventListener('click', closeDialog);

btnOnce.addEventListener('click', async () => {
  try {
    await invokeCommand('approve_admin_item', {
      itemId,
      itemData: null
    });
    closeDialog();
  } catch (err) {
    console.error('Run once failed:', err);
    closeDialog();
  }
});

btnApprove.addEventListener('click', async () => {
  try {
    await invokeCommand('approve_admin_item', {
      itemId,
      itemData: null
    });
    closeDialog();
  } catch (err) {
    console.error('Approve failed:', err);
    closeDialog();
  }
});
