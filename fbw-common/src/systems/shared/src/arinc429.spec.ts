// Copyright (c) 2025 FlyByWire Simulations
// SPDX-License-Identifier: GPL-3.0

import { describe, it, expect } from 'vitest';
import { Arinc429RegisterBetter } from './arinc429_better';

describe('Arinc429RegisterBetter.set', () => {
  it('Correctly construct arinc429 object from 32bit word', () => {
    const rawValue = 2441888944;
    const value = 0b1000110001100010001;
    const ssm = 0b00;
    const sdi = 0b00;
    const label = 0b10110000;
    const parity = 0b1;

    let word = Arinc429RegisterBetter.empty();
    word.set(rawValue);

    expect(word.parity).toBe(parity);
    expect(word.label).toBe(label);
    expect(word.sdi).toBe(sdi);
    expect(word.value).toBe(value);
    expect(word.ssm).toBe(ssm);
    expect(word.rawWord).toBe(rawValue);
  });
});

describe('Arinc429RegisterBetter.fromRawWord', () => {
  it('Correctly construct arinc429 object with method fromRawWord', () => {
    const rawValue = 2441888944;
    const value = 0b1000110001100010001;
    const ssm = 0b00;
    const sdi = 0b00;
    const label = 0b10110000;
    const parity = 0b1;

    let word = Arinc429RegisterBetter.fromRawWord(rawValue);

    expect(word.parity).toBe(parity);
    expect(word.label).toBe(label);
    expect(word.sdi).toBe(sdi);
    expect(word.value).toBe(value);
    expect(word.ssm).toBe(ssm);
    expect(word.rawWord).toBe(rawValue);
  });
});

describe('Arinc429RegisterBetter.setValue', () => {
  it('Correctly update arinc429 data', () => {
    const rawValue = 2441888944;
    const value = 0b1000110001100010001;
    const ssm = 0b00;
    const sdi = 0b00;
    const label = 0b10110000;

    let word = Arinc429RegisterBetter.empty();
    word.set(rawValue);

    const newValue = 0b1000111111100010001;
    word.setValue(newValue);

    expect(word.parity).toBe(0b0);
    expect(word.label).toBe(label);
    expect(word.sdi).toBe(sdi);
    expect(word.value).toBe(newValue);
    expect(word.ssm).toBe(ssm);
    expect(word.rawWord).toBe(301745328);
  });
});

describe('Arinc429RegisterBetter.setBitValue', () => {
  it('Correctly update arinc429 data bit', () => {
    const rawValue = 2441888944;
    const value = 0b1000110001100010001;
    const ssm = 0b00;
    const sdi = 0b00;
    const label = 0b10110000;

    let word = Arinc429RegisterBetter.empty();
    word.set(rawValue);
    word.setBitValue(2, true);

    expect(word.parity).toBe(0b0);
    expect(word.label).toBe(label);
    expect(word.sdi).toBe(sdi);
    expect(word.value).toBe(value+2);
    expect(word.ssm).toBe(ssm);
    expect(word.rawWord).toBe(294407344);
  });
});

describe('Arinc429RegisterBetter.setSsm', () => {
  it('Correctly update arinc429 ssm', () => {
    const rawValue = 2441888944;
    const value = 0b1000110001100010001;
    const sdi = 0b00;
    const label = 0b10110000;

    let word = Arinc429RegisterBetter.empty();
    word.set(rawValue);
    word.setSsm(0b10);

    expect(word.parity).toBe(0b1);
    expect(word.label).toBe(label);
    expect(word.sdi).toBe(sdi);
    expect(word.value).toBe(value);
    expect(word.ssm).toBe(0b10);
    expect(word.rawWord).toBe(3515630768);
  });
});

describe('Arinc429RegisterBetter.setSdi', () => {
  it('Correctly update arinc429 sdi', () => {
    const rawValue = 2441888944;
    const value = 0b1000110001100010001;
    const sdi = 0b00;
    const ssm = 0b00;
    const label = 0b10110000;

    let word = Arinc429RegisterBetter.empty();
    word.set(rawValue);
    word.setSdi(0b01);

    expect(word.parity).toBe(0b1);
    expect(word.label).toBe(label);
    expect(word.sdi).toBe(0b01);
    expect(word.value).toBe(value);
    expect(word.ssm).toBe(ssm);
    expect(word.rawWord).toBe(2441889200);
  });
});

describe('Arinc429RegisterBetter.getBitValue', () => {
  it('Correctly get arinc429 bit value', () => {
    const rawValue = 2441888944;

    let word = Arinc429RegisterBetter.empty();
    word.set(rawValue);

    expect(word.getBitValue(1)).toBe(true);
    expect(word.getBitValue(2)).toBe(false);
  });
});
