import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

export function useTemplateText(key: string) {
  const { t } = useI18n()
  
  return computed(() => {
    const text = t(key) as string
    // Replace [[...]] with {{...}} for display
    return text.replace(/\[\[/g, '{{').replace(/\]\]/g, '}}')
  })
}

