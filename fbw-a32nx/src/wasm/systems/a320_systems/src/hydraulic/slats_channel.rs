use std::cmp::max;

use crate::systems::shared::arinc429::{Arinc429Word, SignStatus};
use systems::hydraulic::command_sensor_unit::{CSUMonitor, CSU};
use systems::shared::{AdirsMeasurementOutputs, LgciuWeightOnWheels, PositionPickoffUnit};

use systems::simulation::{
    InitContext, SimulationElement, SimulationElementVisitor, SimulatorWriter, UpdateContext,
    VariableIdentifier, Write,
};

use uom::si::{angle::degree, f64::*, velocity::knot};

use super::sfcc::SlatFlapControlComputerMisc;

pub struct SlatsChannel {
    slats_fppu_angle_id: VariableIdentifier,
    slat_actual_position_word_id: VariableIdentifier,

    slats_demanded_angle: Angle,
    slats_feedback_angle: Angle,

    csu_monitor: CSUMonitor,

    kts_60: Velocity,
    conf1_slats: Angle,
    slat_lock_low_cas: Velocity,
    slat_lock_high_cas: Velocity,
    slat_lock_low_aoa: Angle,
    slat_lock_high_aoa: Angle,

    slat_lock_command_angle: Angle,
    slat_baulk_engaged: bool,
    slat_alpha_lock_engaged: bool,
}

impl SlatsChannel {
    const CONF1_SLATS_DEGREES: f64 = 222.27; //deg
    const SLAT_LOCK_ACTIVE_SPEED_KNOTS: f64 = 60.; //kts
    const SLAT_LOCK_LOW_SPEED_KNOTS: f64 = 148.; //deg
    const SLAT_LOCK_HIGH_SPEED_KNOTS: f64 = 154.; //deg
    const SLAT_LOCK_LOW_ALPHA_DEGREES: f64 = 8.5; //deg
    const SLAT_LOCK_HIGH_ALPHA_DEGREES: f64 = 7.6; //deg

    pub fn new(context: &mut InitContext, num: u8) -> Self {
        Self {
            slats_fppu_angle_id: context.get_identifier("SLATS_FPPU_ANGLE".to_owned()),
            slat_actual_position_word_id: context
                .get_identifier(format!("SFCC_{num}_SLAT_ACTUAL_POSITION_WORD")),

            slats_demanded_angle: Angle::new::<degree>(0.),
            slats_feedback_angle: Angle::new::<degree>(0.),

            csu_monitor: CSUMonitor::new(context),

            kts_60: Velocity::new::<knot>(Self::SLAT_LOCK_ACTIVE_SPEED_KNOTS),
            conf1_slats: Angle::new::<degree>(Self::CONF1_SLATS_DEGREES),
            slat_lock_low_cas: Velocity::new::<knot>(Self::SLAT_LOCK_LOW_SPEED_KNOTS),
            slat_lock_high_cas: Velocity::new::<knot>(Self::SLAT_LOCK_HIGH_SPEED_KNOTS),
            slat_lock_low_aoa: Angle::new::<degree>(Self::SLAT_LOCK_LOW_ALPHA_DEGREES),
            slat_lock_high_aoa: Angle::new::<degree>(Self::SLAT_LOCK_HIGH_ALPHA_DEGREES),

            slat_lock_command_angle: Angle::new::<degree>(0.),
            slat_baulk_engaged: false,
            slat_alpha_lock_engaged: false,
        }
    }

    // Returns a slat demanded angle in FPPU reference degree (feedback sensor)
    fn demanded_slats_fppu_angle_from_conf(
        csu_monitor: &CSUMonitor,
        last_demanded: Angle,
    ) -> Angle {
        match csu_monitor.get_current_detent() {
            CSU::Conf0 => Angle::new::<degree>(0.),
            CSU::Conf1 => Angle::new::<degree>(222.27),
            CSU::Conf2 => Angle::new::<degree>(272.27),
            CSU::Conf3 => Angle::new::<degree>(272.27),
            CSU::ConfFull => Angle::new::<degree>(334.16),
            _ => last_demanded,
        }
    }

    fn update_slat_alpha_lock(
        &mut self,
        adirs: &impl AdirsMeasurementOutputs,
        lgciu: &impl LgciuWeightOnWheels,
        // flaps_handle: &impl FlapsHandle,
        // aoa: Option<Angle>,
        // cas: Option<Velocity>,
    ) {
        let aoa1 = adirs.angle_of_attack(1).normal_value();
        let aoa2 = adirs.angle_of_attack(2).normal_value();
        let aoa = match (aoa1, aoa2) {
            (Some(aoa1), Some(aoa2)) => Some(Angle::min(aoa1, aoa2)),
            (Some(aoa1), None) => Some(aoa1),
            (None, Some(aoa2)) => Some(aoa2),
            (None, None) => None,
        };

        if !(cas.unwrap_or_default() >= self.kts_60 || lgciu.left_and_right_gear_extended(false)) {
            // println!("Exiting update_slat_lock");
            self.slat_alpha_lock_engaged = false;
            return;
        }

        let current_detent = self.csu_monitor.get_current_detent();

        match aoa {
            Some(aoa)
                if aoa > self.slat_lock_high_aoa
                    && current_detent == CSU::Conf0
                    && SlatFlapControlComputerMisc::in_or_above_enlarged_target_range(
                        self.slats_feedback_angle,
                        self.conf1_slats,
                    ) =>
            {
                // println!("S2");
                self.slat_alpha_lock_engaged = true;
            }
            Some(_) if current_detent == CSU::OutOfDetent => {
                // println!("S3");
                // self.slat_alpha_lock_engaged = self.slat_alpha_lock_engaged;
            }
            Some(aoa)
                if aoa < self.slat_lock_low_aoa
                    && current_detent == CSU::Conf0
                    && self.slat_alpha_lock_engaged =>
            {
                // println!("S4");
                self.slat_alpha_lock_engaged = false;
            }
            None if current_detent == CSU::Conf0 => {
                // println!("S6");
                self.slat_alpha_lock_engaged = false;
            }
            // Verify if it shall be false or true!
            _ => {
                // println!("S8");
                self.slat_alpha_lock_engaged = false;
            } // panic!(
              //     "Missing case update_slat_lock! {} {}.",
              //     self.cas.unwrap().get::<knot>(),
              //     self.aoa.unwrap().get::<degree>()
              // ),
        }

        if self.slat_alpha_lock_engaged {
            self.slat_lock_command_angle = self.conf1_slats
        }

        // println!(
        //     "CAS_MAX {}\tAOA {}",
        //     cas.unwrap_or_default().get::<knot>(),
        //     aoa.unwrap_or_default().get::<degree>()
        // );
    }

    fn update_slat_baulk(
        &mut self,
        adirs: &impl AdirsMeasurementOutputs,
        lgciu: &impl LgciuWeightOnWheels,
        // flaps_handle: &impl FlapsHandle,
        // _aoa: Option<Angle>,
        // cas: Option<Velocity>,
    ) {
        let cas1 = adirs.computed_airspeed(1).normal_value();
        let cas2 = adirs.computed_airspeed(2).normal_value();
        let cas = match (cas1, cas2) {
            (Some(cas1), Some(cas2)) => Some(Velocity::max(cas1, cas2)),
            (Some(cas1), None) => Some(cas1),
            (None, Some(cas2)) => Some(cas2),
            (None, None) => None,
        };

        if cas.is_none() {
            self.slat_baulk_engaged = false;
            return;
        }

        if !(cas.unwrap_or_default() >= self.kts_60 || lgciu.left_and_right_gear_extended(false)) {
            // println!("Exiting update_slat_lock");
            self.slat_baulk_engaged = false;
            return;
        }

        let current_detent = self.csu_monitor.get_current_detent();

        match cas {
            Some(cas)
                if cas < self.slat_lock_low_cas
                    && current_detent == CSU::Conf0
                    && SlatFlapControlComputerMisc::in_or_above_enlarged_target_range(
                        self.slats_feedback_angle,
                        self.conf1_slats,
                    ) =>
            {
                // println!("S9");
                self.slat_baulk_engaged = true;
            }
            Some(_) if current_detent == CSU::OutOfDetent => {
                // println!("S10");
                // self.slat_baulk_engaged = self.slat_baulk_engaged;
            }
            Some(cas)
                if cas > self.slat_lock_high_cas
                    && current_detent == CSU::Conf0
                    && self.slat_baulk_engaged =>
            {
                // println!("S11");
                self.slat_baulk_engaged = false;
            }
            None if current_detent == CSU::Conf0 => {
                // println!("S12");
                self.slat_baulk_engaged = false;
            }
            // Verify if it shall be false or true!
            _ => {
                // println!("S13");
                self.slat_baulk_engaged = false;
            } // panic!(
              //     "Missing case update_slat_lock! {} {}.",
              //     self.cas.unwrap().get::<knot>(),
              //     self.aoa.unwrap().get::<degree>()
              // ),
        }

        if self.slat_baulk_engaged {
            self.slat_lock_command_angle = self.conf1_slats
        }

        // println!(
        //     "CAS_MAX {}\tAOA {}",
        //     cas.unwrap_or_default().get::<knot>(),
        //     aoa.unwrap_or_default().get::<degree>()
        // );
    }

    fn generate_slat_angle(
        &mut self,
        adirs: &impl AdirsMeasurementOutputs,
        lgciu: &impl LgciuWeightOnWheels,
    ) -> Angle {
        // self.update_slat_alpha_lock(lgciu, flaps_handle, aoa, cas);
        self.update_slat_baulk(adirs, lgciu);

        // self.slat_retraction_inhibited = self.slat_alpha_lock_engaged || self.slat_baulk_engaged;

        // if self.slat_retraction_inhibited {
        //     return self.slat_lock_command_angle;
        // }

        Self::demanded_slats_fppu_angle_from_conf(&self.csu_monitor, self.slats_demanded_angle)
    }

    pub fn update(
        &mut self,
        context: &UpdateContext,
        slats_feedback: &impl PositionPickoffUnit,
        adirs: &impl AdirsMeasurementOutputs,
        lgciu: &impl LgciuWeightOnWheels,
    ) {
        self.csu_monitor.update(context);
        self.slats_demanded_angle = self.generate_slat_angle(adirs, lgciu);

        self.slats_feedback_angle = slats_feedback.angle();
    }

    pub fn get_demanded_angle(&self) -> Angle {
        self.slats_demanded_angle
    }

    pub fn get_feedback_angle(&self) -> Angle {
        self.slats_feedback_angle
    }

    pub fn get_csu_monitor(&self) -> &CSUMonitor {
        &self.csu_monitor
    }

    fn slat_actual_position_word(&self) -> Arinc429Word<f64> {
        Arinc429Word::new(
            self.slats_feedback_angle.get::<degree>(),
            SignStatus::NormalOperation,
        )
    }
}
impl SimulationElement for SlatsChannel {
    fn accept<T: SimulationElementVisitor>(&mut self, visitor: &mut T) {
        self.csu_monitor.accept(visitor);
        visitor.visit(self);
    }

    fn write(&self, writer: &mut SimulatorWriter) {
        writer.write(&self.slats_fppu_angle_id, self.slats_feedback_angle);

        writer.write(
            &self.slat_actual_position_word_id,
            self.slat_actual_position_word(),
        );
    }
}
