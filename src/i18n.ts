import { createI18n } from 'vue-i18n'
import en from './locales/en'
import ru from './locales/ru'

// Get locale from localStorage or default to 'en'
const savedLocale = localStorage.getItem('locale')
const initialLocale = (savedLocale === 'en' || savedLocale === 'ru') ? savedLocale : 'en'

const i18n = createI18n({
  legacy: false,
  locale: initialLocale,
  fallbackLocale: 'en',
  messages: {
    en,
    ru
  },
  missingWarn: false,
  fallbackWarn: false,
  warnHtmlMessage: false
})

export default i18n

