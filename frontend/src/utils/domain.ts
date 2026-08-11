export interface ParsedDomain {
    name: string
    port: number | null
}

export function parseDomainLine(line: string): ParsedDomain {
    const s = line.trim()
    const idx = s.lastIndexOf(':')
    if (idx > 0 && /^\d+$/.test(s.slice(idx + 1))) {
        return { name: s.slice(0, idx), port: parseInt(s.slice(idx + 1), 10) }
    }
    return { name: s, port: null }
}

export function parseDomains(input: string): ParsedDomain[] {
    return input
        .split('\n')
        .map(s => s.trim())
        .filter(s => s.length > 0)
        .map(parseDomainLine)
}
