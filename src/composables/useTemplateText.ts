import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

export function useTemplateText(key: string) {
  const { t } = useI18n()
  
  return computed(() => {
    const text = t(key) as string
    // Replace [[...]] with {{...}} for display
    // Escape braces as HTML entities to prevent Vue from interpreting them as expressions
    return text.replace(/\[\[/g, '&#123;&#123;').replace(/\]\]/g, '&#125;&#125;')
  })
}

