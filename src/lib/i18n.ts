import { browser } from '$app/environment';
import { init, register, getLocaleFromNavigator, locale } from 'svelte-i18n';

// Registrar los idiomas. 
// Usamos "register" en lugar de import directo para habilitar la carga perezosa 
// o manejar los archivos JSON por separado asíncronamente si crece el alcance.
register('en', () => import('./locales/en.json'));
register('es', () => import('./locales/es.json'));

// Idiomas de fallback
const fallbackLocale = 'en';

export function setupI18n() {
	let initialLocale = browser ? getLocaleFromNavigator() : fallbackLocale;

	// Normalizar el locale para que coincida con nuestras traducciones (solo parte principal)
	// Ej: es-MX -> es, en-US -> en
	if (initialLocale && initialLocale.includes('-')) {
		initialLocale = initialLocale.split('-')[0];
	}

	init({
		fallbackLocale,
		initialLocale: initialLocale || fallbackLocale,
	});
}
