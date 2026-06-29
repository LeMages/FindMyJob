import { nextTick } from 'vue'

export function insertBullet(
  el: HTMLTextAreaElement,
  value: string,
  onUpdate: (newValue: string) => void
): void {
  const start = el.selectionStart
  const end = el.selectionEnd
  const hasSelection = start !== end
  const toInsert = hasSelection
    ? '• '
    : start > 0 && value[start - 1] !== '\n'
    ? '\n• '
    : '• '
  onUpdate(value.slice(0, start) + toInsert + value.slice(end))
  const newCursor = start + toInsert.length
  nextTick(() => {
    el.setSelectionRange(newCursor, newCursor)
    el.focus()
  })
}
