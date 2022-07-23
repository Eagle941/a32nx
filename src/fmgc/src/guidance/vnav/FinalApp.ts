//  Copyright (c) 2022 FlyByWire Simulations
//  SPDX-License-Identifier: GPL-3.0

import { GuidanceController } from '@fmgc/guidance/GuidanceController';
import { DecelPathCharacteristics } from '@fmgc/guidance/vnav/descent/DecelPathBuilder';

export interface FinalAppGuidanceParameters {
    targetVerticalSpeed: Degrees,
    /** pressure altitude */
    targetAltitude: Feet,
}

export class FinalAppGuidance {
    private lastRnavAppSelected = false;

    private lastFinalCanEngage = false;

    private onProfile = false;

    private lastVdev = 0;

    private guidanceParams: FinalAppGuidanceParameters;

    update(profile: DecelPathCharacteristics, guidanceController: GuidanceController, finalActive: boolean) {
        let finalCanEngage = false;
        let vDev = 0;

        const rnavAppSelected = !!(profile?.finalDescent?.validForGuidance && profile.finalDescent.nonPrecision);

        if (rnavAppSelected) {
            const anchorLegIndex = profile.finalDescent.anchorLegIndex;

            let anchorDtg = guidanceController.activeLegCompleteLegPathDtg;
            for (let i = guidanceController.activeLegIndex + 1; i <= anchorLegIndex; i++) {
                const leg = guidanceController.activeGeometry.legs.get(i);
                anchorDtg += leg.distance;
            }

            const targetIndicatedAlt = profile.finalDescent.finalAltitude + (anchorDtg * 1852 / 0.3048) * Math.tan(-profile.finalDescent.fpa * Math.PI / 180);

            const baroCorrection = SimVar.GetSimVarValue('KOHLSMAN SETTING MB:1', 'millibars');

            const currentPressureAlt = SimVar.GetSimVarValue('INDICATED ALTITUDE:3', 'feet');

            const targetAltitude = targetIndicatedAlt + 145442.15 * (1 - (baroCorrection / 1013.25) ** 0.190263);

            const gs = SimVar.GetSimVarValue('GPS GROUND SPEED', 'knots');

            const targetVerticalSpeed = gs / 60 * 1852 / 0.3048 * Math.tan(profile.finalDescent.fpa * Math.PI / 180);

            vDev = currentPressureAlt - targetAltitude;

            this.onProfile = this.onProfile || vDev > (-gs / 6);

            this.guidanceParams = {
                targetAltitude: this.onProfile ? targetAltitude : Math.min(targetAltitude, currentPressureAlt),
                targetVerticalSpeed,
            };

            // TODO check logic for descent below MDA etc.

            // TODO check limits
            finalCanEngage = Math.abs(vDev) < 150 && guidanceController.lastCrosstrackError < 0.2 && guidanceController.activeLegIndex <= profile.finalDescent.anchorLegIndex;
        }

        this.onProfile = this.onProfile && finalActive;

        if (this.lastRnavAppSelected !== rnavAppSelected) {
            this.lastRnavAppSelected = rnavAppSelected;
            SimVar.SetSimVarValue('L:A32NX_FG_RNAV_APP_SELECTED', 'boolean', rnavAppSelected);
        }

        if (this.lastFinalCanEngage !== finalCanEngage) {
            this.lastFinalCanEngage = finalCanEngage;
            SimVar.SetSimVarValue('L:A32NX_FG_FINAL_CAN_ENGAGE', 'boolean', finalCanEngage);
        }

        // TODO ARINC label
        if (Math.abs(this.lastVdev - vDev) >= 1) {
            this.lastVdev = vDev;
            SimVar.SetSimVarValue('L:A32NX_FM_VDEV', 'number', vDev);
        }
    }

    getGuidanceParameters(_profile: DecelPathCharacteristics, _guidanceController: GuidanceController): FinalAppGuidanceParameters {
        return this.guidanceParams;
    }
}
