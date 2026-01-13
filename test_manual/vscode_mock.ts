
export enum DiagnosticSeverity {
    Error = 0,
    Warning = 1,
    Information = 2,
    Hint = 3
}

export class Position {
    constructor(public line: number, public character: number) { }
}

export class Range {
    constructor(public start: Position, public end: Position) { }
}

export class Diagnostic {
    source?: string;
    constructor(public range: Range, public message: string, public severity: DiagnosticSeverity) { }
}

export class Uri {
    static file(path: string): Uri {
        return new Uri(path);
    }
    constructor(public fsPath: string) { }
}
