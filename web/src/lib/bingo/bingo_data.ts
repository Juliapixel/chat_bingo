export interface BingoSquare {
    text: string;
    selected: boolean;
}

export class BingoCardData {
    data: Array<BingoSquare>;
    readonly size: number;

    constructor(size: number, data?: Array<BingoSquare>) {
        this.size = size
        if (data !== undefined) {
            if (data.length != size * size) {
                throw new Error("data length and bingo board size don't match")
            } else {
                this.data = data;
            }
        } else {
            this.data = Array.from({length: size * size}, () => { return { text: "", selected: false } })
        }
    }

    getData(x: number, y: number): BingoSquare | undefined {
        return this.data[this.size * x + y]
    }

    setData(x: number, y: number, data: BingoSquare) {
        this.data[this.size * x + y] = data;
    }
}
