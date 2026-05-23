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
	let initialLocale = fallbackLocale;

	if (browser) {
		const savedLocale = window.localStorage.getItem('anvilcraft_locale');
		if (savedLocale) {
			initialLocale = savedLocale;
		} else {
			const navLocale = getLocaleFromNavigator();
			if (navLocale) {
				initialLocale = navLocale.includes('-') ? navLocale.split('-')[0] : navLocale;
			}
		}
	}

	// Ensure the initial locale is supported, otherwise use fallback
	const supportedLocales = ['en', 'es'];
	if (!initialLocale || !supportedLocales.includes(initialLocale)) {
		initialLocale = fallbackLocale;
	}

	init({
		fallbackLocale,
		initialLocale,
	});
}
