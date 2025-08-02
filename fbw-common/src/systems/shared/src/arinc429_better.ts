const LABEL_MASK = ((0b1 << 8) - 1);
const SDI_MASK = ((0b1 << 2) - 1);
const VALUE_MASK = ((0b1 << 19) - 1);
const SSM_MASK = ((0b1 << 2) - 1);
const PARITY_MASK = ((0b1 << 1) - 1);

const SDI_SHIFT = 8;
const VALUE_SHIFT = 10;
const SSM_SHIFT = 29;
const PARITY_SHIFT = 31;

// the arinc word is represented with the equivalent f64 format.
export class Arinc429RegisterBetter {
  protected static u32View = new Uint32Array(1);

  rawWord = 0;

  label: number;
  sdi: number;
  value: number;
  ssm: number;
  parity: number;

  static empty() {
    return new Arinc429RegisterBetter();
  }

  static fromRawWord(rawWord: number): Arinc429RegisterBetter {
    return new Arinc429RegisterBetter().set(rawWord);
  }

  protected constructor() {
    this.set(0);
  }

  set(rawWord: number): Arinc429RegisterBetter {
    this.rawWord = rawWord;
    Arinc429RegisterBetter.u32View[0] = (rawWord & 0xffffffff) >>> 0;

    this.label = Arinc429RegisterBetter.u32View[0] & LABEL_MASK;
    this.sdi = (Arinc429RegisterBetter.u32View[0] >>> SDI_SHIFT) & SDI_MASK;
    this.value = (Arinc429RegisterBetter.u32View[0] >>> VALUE_SHIFT) & VALUE_MASK;
    this.ssm = (Arinc429RegisterBetter.u32View[0] >>> SSM_SHIFT) & SDI_MASK;
    this.parity = (Arinc429RegisterBetter.u32View[0] >>> PARITY_SHIFT) & PARITY_MASK;

    return this;
  }

  private updateRawWord(): void {
    this.rawWord = (this.label & LABEL_MASK) | ((this.sdi & SDI_MASK) << SDI_SHIFT) | ((this.value & VALUE_MASK) << VALUE_SHIFT) | ((this.ssm & SDI_MASK) << SSM_SHIFT) | ((this.parity & PARITY_MASK) << PARITY_SHIFT);
    this.rawWord >>>= 0;
  }

  private calculateParityBit(): number {
    let value_u32 = (this.label & LABEL_MASK) | ((this.sdi & SDI_MASK) << SDI_SHIFT) | ((this.value & VALUE_MASK) << VALUE_SHIFT) | ((this.ssm & SDI_MASK) << SSM_SHIFT);
    // Taken from https://graphics.stanford.edu/~seander/bithacks.html#ParityParallel
    let v = value_u32;
    v ^= v >> 16;
    v ^= v >> 8;
    v ^= v >> 4;
    v &= 0xf;
    const parity_even = (0x6996 >> v) & 1;
    const parity_odd = (parity_even + 1) % 2;
    return parity_odd;
  }

  setBitValue(bit: number, value: boolean): void {
    // LSB is 1
    if (value) {
      this.value |= 1 << (bit - 1);
    } else {
      this.value &= ~(1 << (bit - 1));
    }
    this.parity = this.calculateParityBit();
    this.updateRawWord();
  }

  setValue(value: number): void {
    this.value = value;
    this.parity = this.calculateParityBit();
    this.updateRawWord();
  }

  setSsm(ssm: number): void {
    this.ssm = ssm;
    this.parity = this.calculateParityBit();
    this.updateRawWord();
  }

  setSdi(sdi: number): void {
    this.sdi = sdi;
    this.parity = this.calculateParityBit();
    this.updateRawWord();
  }

  setLabel(label: number): void {
    this.label = label;
    this.parity = this.calculateParityBit();
    this.updateRawWord();
  }

  setFromSimVar(name: string): Arinc429RegisterBetter {
    return this.set(SimVar.GetSimVarValue(name, 'number'));
  }

  writeToSimVar(name: string): void {
    SimVar.SetSimVarValue(name, 'string', this.rawWord.toString());
  }

  getBitValue(bit: number): boolean {
    // LSB is 1
    return ((this.value >> (bit - 1)) & 1) !== 0;
  }

  getSsm(): number {
    return this.ssm
  }
}

export enum Arinc429DiscreteSignStatusMatrix {
  NormalOperation = 0b00,
  NoComputedData = 0b01,
  FunctionalTest = 0b10,
  FailureWarning = 0b11,
}

export class Arinc429DiscreteDataWord extends Arinc429RegisterBetter {
  static fromRawWord(rawWord: number): Arinc429DiscreteDataWord {
    return new Arinc429RegisterBetter().set(rawWord);
  }

  setSsm(ssm: Arinc429DiscreteSignStatusMatrix): void {
    super.setSsm(ssm as number);
  }

  getSsm(): Arinc429DiscreteSignStatusMatrix {
    return super.getSsm() as Arinc429DiscreteSignStatusMatrix;
  }
}

export enum Arinc429BNRSignStatusMatrix {
  FailureWarning = 0b00,
  NoComputedData = 0b01,
  FunctionalTest = 0b10,
  NormalOperation = 0b11,
}

export class Arinc429BNRWord extends Arinc429RegisterBetter {
  static fromRawWord(rawWord: number): Arinc429BNRWord {
    return new Arinc429RegisterBetter().set(rawWord);
  }

  setSsm(ssm: Arinc429BNRSignStatusMatrix): void {
    super.setSsm(ssm as number);
  }

  getSsm(): Arinc429BNRSignStatusMatrix {
    return super.getSsm() as Arinc429BNRSignStatusMatrix;
  }
}

export enum Arinc429BCDSignStatusMatrix {
  PlusNorthEastRightToAbove = 0b00,
  NoComputedData = 0b01,
  FunctionalTest = 0b10,
  MinusSouthWestLeftFromBelow = 0b11,
}

export class Arinc429BCDWord extends Arinc429RegisterBetter {
  static fromRawWord(rawWord: number): Arinc429BCDWord {
    return new Arinc429RegisterBetter().set(rawWord);
  }

  public constructor(label: number, sdi: number, value: number, ssm: Arinc429BCDSignStatusMatrix) {
    super();

    Arinc429RegisterBetter.u32View[0] = (label & 0xffffffff) >>> 0;
    this.label = Arinc429RegisterBetter.u32View[0] & LABEL_MASK;

    Arinc429RegisterBetter.u32View[0] = (sdi & 0xffffffff) >>> 0;
    this.sdi = Arinc429RegisterBetter.u32View[0] & SDI_MASK;

    Arinc429RegisterBetter.u32View[0] = ((ssm as number) & 0xffffffff) >>> 0;
    this.ssm =  Arinc429RegisterBetter.u32View[0] & SSM_MASK;

    Arinc429RegisterBetter.u32View[0] = (value & 0xffffffff) >>> 0;
    this.setValue(Arinc429RegisterBetter.u32View[0] & VALUE_MASK);
  }

  setSsm(ssm: Arinc429BCDSignStatusMatrix): void {
    super.setSsm(ssm as number);
  }

  getSsm(): Arinc429BCDSignStatusMatrix {
    return super.getSsm() as Arinc429BCDSignStatusMatrix;
  }
}
