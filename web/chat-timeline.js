(function (root) {
  function messageKey(row) {
    return String(row && (row.id || [row.created_at || '', row.role || '', row.content || ''].join('\n')));
  }

  function timestamp(row) {
    const value = Date.parse((row && row.created_at) || '');
    return Number.isFinite(value) ? value : null;
  }

  function compareRows(left, right) {
    const leftTime = timestamp(left);
    const rightTime = timestamp(right);
    if (leftTime !== null && rightTime !== null && leftTime !== rightTime) return leftTime - rightTime;
    if (leftTime !== null && rightTime === null) return -1;
    if (leftTime === null && rightTime !== null) return 1;
    return messageKey(left).localeCompare(messageKey(right));
  }

  function insertionIndex(rows, row) {
    const index = rows.findIndex((current) => compareRows(row, current) < 0);
    return index < 0 ? rows.length : index;
  }

  function createLiveTurn() {
    let currentText = '';
    let workActive = false;
    return {
      appendText(delta) { currentText += String(delta || ''); },
      activeText() { return currentText; },
      beginTools() {
        const completed = currentText.trim();
        currentText = '';
        workActive = true;
        return completed ? { content: completed, workActive } : { content: '', workActive };
      },
      finishText() {
        const completed = currentText.trim();
        currentText = '';
        return completed;
      },
      isWorkActive() { return workActive; },
    };
  }

  root.CommandBlockTimeline = { messageKey, compareRows, insertionIndex, createLiveTurn };
})(globalThis);
