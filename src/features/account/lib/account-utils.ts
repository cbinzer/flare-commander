export function getBeginningLetters(name: string) {
  if (!name) {
    return 'XX';
  }

  const words = name.split(' ');
  if (words.length > 1) {
    const firstLetter = words[0][0]?.toUpperCase() ?? 'X';
    const secondLetter = words[1][0]?.toUpperCase() ?? 'X';
    return firstLetter + secondLetter;
  }

  const firstLetter = words[0][0]?.toUpperCase() ?? 'X';
  const secondLetter = words[0][1]?.toUpperCase() ?? 'X';
  return firstLetter + secondLetter;
}
