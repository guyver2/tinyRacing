/**
 * Utility functions for working with country flags and nationality codes
 */

/**
 * Maps nationality terms (e.g., 'french', 'german') to ISO country codes
 */
function getCountryCodeFromNationality(nationality: string): string | null {
  // Map terms like 'french', 'german', 'dutch', etc. to ISO country codes, case-insensitive
  const nationalityToCountry: Record<string, string> = {
    french: 'fr',
    german: 'de',
    dutch: 'nl',
    british: 'gb',
    english: 'gb',
    spanish: 'es',
    italian: 'it',
    swiss: 'ch',
    belgian: 'be',
    austrian: 'at',
    monégasque: 'mc',
    monegasque: 'mc',
    brazilian: 'br',
    argentine: 'ar',
    mexican: 'mx',
    canadian: 'ca',
    australian: 'au',
    'new zealander': 'nz',
    japanese: 'jp',
    chinese: 'cn',
    korean: 'kr',
    'south korean': 'kr',
    indian: 'in',
    russian: 'ru',
    finnish: 'fi',
    swedish: 'se',
    norwegian: 'no',
    danish: 'dk',
    polish: 'pl',
    czech: 'cz',
    hungarian: 'hu',
    romanian: 'ro',
    turkish: 'tr',
    'south african': 'za',
    portuguese: 'pt',
    greek: 'gr',
    irish: 'ie',
    thai: 'th',
    malaysian: 'my',
    singaporean: 'sg',
    indonesian: 'id',
    american: 'us',
    'us-american': 'us',
    usa: 'us',
    briton: 'gb',
    scottish: 'gb',
    welsh: 'gb',
    'northern irish': 'gb',
  };
  // Normalize input
  const key = nationality.trim().toLowerCase();
  return nationalityToCountry[key] || null;
}

/**
 * Maps country names to ISO 3166-1 alpha-2 country codes
 */
export function getCountryCode(countryName: string): string | null {
  // Map country names to ISO 3166-1 alpha-2 country codes
  const countryCodes: Record<string, string> = {
    'United Kingdom': 'gb',
    UK: 'gb',
    'United States': 'us',
    USA: 'us',
    US: 'us',
    France: 'fr',
    Germany: 'de',
    Italy: 'it',
    Spain: 'es',
    Netherlands: 'nl',
    Belgium: 'be',
    Switzerland: 'ch',
    Austria: 'at',
    Monaco: 'mc',
    Brazil: 'br',
    Argentina: 'ar',
    Mexico: 'mx',
    Canada: 'ca',
    Australia: 'au',
    'New Zealand': 'nz',
    Japan: 'jp',
    China: 'cn',
    'South Korea': 'kr',
    Korea: 'kr',
    India: 'in',
    Russia: 'ru',
    Finland: 'fi',
    Sweden: 'se',
    Norway: 'no',
    Denmark: 'dk',
    Poland: 'pl',
    'Czech Republic': 'cz',
    Hungary: 'hu',
    Romania: 'ro',
    Turkey: 'tr',
    'South Africa': 'za',
    Portugal: 'pt',
    Greece: 'gr',
    Ireland: 'ie',
    Thailand: 'th',
    Malaysia: 'my',
    Singapore: 'sg',
    Indonesia: 'id',
    Philippines: 'ph',
    Vietnam: 'vn',
    UAE: 'ae',
    'United Arab Emirates': 'ae',
    'Saudi Arabia': 'sa',
    Qatar: 'qa',
    Bahrain: 'bh',
    Kuwait: 'kw',
    Oman: 'om',
    Israel: 'il',
    Lebanon: 'lb',
    Jordan: 'jo',
    Egypt: 'eg',
    Morocco: 'ma',
    Tunisia: 'tn',
    Algeria: 'dz',
    Nigeria: 'ng',
    Kenya: 'ke',
    Ghana: 'gh',
    Senegal: 'sn',
    'Ivory Coast': 'ci',
    "Côte d'Ivoire": 'ci',
    Cameroon: 'cm',
    Chile: 'cl',
    Colombia: 'co',
    Peru: 'pe',
    Venezuela: 've',
    Ecuador: 'ec',
    Uruguay: 'uy',
    Paraguay: 'py',
    Bolivia: 'bo',
  };

  // Try exact match first
  if (countryCodes[countryName]) {
    return countryCodes[countryName];
  }

  // Try case-insensitive match
  const normalizedName = countryName.trim();
  for (const [key, code] of Object.entries(countryCodes)) {
    if (key.toLowerCase() === normalizedName.toLowerCase()) {
      return code;
    }
  }
  // try with the nationality
  const nationalityCode = getCountryCodeFromNationality(countryName);
  if (nationalityCode) {
    return nationalityCode;
  }

  return null;
}

// Store flag URLs in a module-level cache
const flagUrlCache = new Map<string, string>();

/**
 * Gets the URL for a country flag SVG based on the country code
 * @param countryCode ISO 3166-1 alpha-2 country code (e.g., 'us', 'gb', 'fr')
 * @returns URL to the flag SVG file
 */
export function getFlagUrl(countryCode: string): string {
  const code = countryCode.toLowerCase();

  // Check cache first
  if (flagUrlCache.has(code)) {
    return flagUrlCache.get(code)!;
  }

  // Flags are in public/assets/country-flags/svg/
  // In Vite, public folder files are served from root
  const url = `/assets/country-flags/svg/${code}.svg`;
  flagUrlCache.set(code, url);
  return url;
}
