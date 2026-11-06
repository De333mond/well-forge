export interface CpsGrid {
  rows: number
  columns: number
  values: (number | null)[]
  xMin: number
  xMax: number
  yMin: number
  yMax: number
}

export function parseCps(content: string): CpsGrid {
  const surface = content.split(/^\s*->Surface\s*$/im)
  if (surface.length !== 2) throw new Error('Не найден единственный раздел Surface.')
  const header = surface[0]!
  const dimensions = header.match(/^\s*FSNROW\s+(\d+)\s+(\d+)\s*$/im)
  const ascii = header.match(/^\s*FSASCI\s+\S+\s+\S+\s+\S+\s+\S+\s+(\S+)\s*$/im)
  const bounds = header.match(/^\s*FSLIMI[ \t]+([^\r\n]+)$/im)
  if (!dimensions || !ascii || !bounds) throw new Error('В заголовке отсутствуют размеры, координаты сетки или маркер пустого значения.')
  const rows = Number(dimensions[1])
  const columns = Number(dimensions[2])
  const missing = Number(ascii[1])
  const boundValues = bounds[1]!.trim().split(/\s+/).map(Number)
  const [xMin, xMax, yMin, yMax] = boundValues
  if (boundValues.length < 4 || xMin === undefined || xMax === undefined || yMin === undefined || yMax === undefined || ![xMin, xMax, yMin, yMax].every(Number.isFinite) || xMax < xMin || yMax < yMin || (columns > 1 && xMax === xMin) || (rows > 1 && yMax === yMin)) {
    throw new Error('Некорректные координаты в FSLIMI.')
  }
  if (!Number.isSafeInteger(rows) || !Number.isSafeInteger(columns) || rows < 1 || columns < 1 || rows * columns > 1_000_000 || !Number.isFinite(missing)) {
    throw new Error('Некорректные размеры сетки или маркер пустого значения.')
  }
  const tokens = surface[1]!.trim().split(/\s+/)
  if (tokens.length !== rows * columns) throw new Error(`Ожидается ${rows * columns} значений, найдено ${tokens.length}.`)
  const values: (number | null)[] = Array(rows * columns).fill(null)
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index]!
    const value = Number(token)
    if (!token || !Number.isFinite(value)) throw new Error(`Некорректное значение в позиции ${index + 1}.`)
    const column = Math.floor(index / rows)
    const row = rows - 1 - index % rows
    values[row * columns + column] = value === missing ? null : value
  }
  return { rows, columns, values, xMin, xMax, yMin, yMax }
}
