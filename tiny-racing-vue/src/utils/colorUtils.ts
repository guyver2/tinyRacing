/**
 * Converts a hex color to its pastel version
 * Pastel colors are created by mixing the color with white (increasing lightness)
 * @param hex - Hex color string (e.g., "#FF0000" or "FF0000")
 * @param mixRatio - Ratio of white to mix (0-1), higher = more pastel. Default 0.7
 * @returns Pastel hex color string
 */
export function hexToPastel(hex: string, mixRatio: number = 0.7): string {
  if (!hex) {
    // Return a default pastel color if hex is empty
    return '#f0f0f0';
  }

  // Remove # if present
  hex = hex.replace('#', '');

  // Validate hex color format (should be 6 characters)
  if (hex.length !== 6 || !/^[0-9A-Fa-f]{6}$/.test(hex)) {
    // If invalid, return a default pastel color
    return '#f0f0f0';
  }

  // Parse RGB values
  const r = parseInt(hex.substring(0, 2), 16);
  const g = parseInt(hex.substring(2, 4), 16);
  const b = parseInt(hex.substring(4, 6), 16);

  // Mix with white (255, 255, 255) using the mix ratio
  const pastelR = Math.round(r + (255 - r) * mixRatio);
  const pastelG = Math.round(g + (255 - g) * mixRatio);
  const pastelB = Math.round(b + (255 - b) * mixRatio);

  // Convert back to hex
  const toHex = (n: number) => {
    const hex = n.toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };

  return `#${toHex(pastelR)}${toHex(pastelG)}${toHex(pastelB)}`;
}
