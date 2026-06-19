import AnsiToHtml from 'ansi-to-html';

const converter = new AnsiToHtml({
  fg: '#d4d4d4',
  bg: '#1e293b',
  newline: false,
  escapeXML: true, // Importante para evitar que el navegador interprete tags en el log
});

export function convertAnsiToHtml(text: string): string {
  return converter.toHtml(text);
}

export function hasAnsiCodes(text: string): boolean {
  return /\x1b\[[0-9;]*[a-zA-Z]/.test(text);
}

