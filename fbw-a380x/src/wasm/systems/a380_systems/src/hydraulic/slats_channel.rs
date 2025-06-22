use crate::systems::shared::arinc429::{Arinc429Word, SignStatus};
use systems::hydraulic::command_sensor_unit::{CSUMonitor, CSU};
use systems::shared::{AdirsMeasurementOutputs, PositionPickoffUnit};

use systems::simulation::{
    InitContext, SimulationElement, SimulatorWriter, UpdateContext, VariableIdentifier, Write,
};

use uom::si::length::foot;
use uom::si::{angle::degree, f64::*, velocity::knot};
use uom::ConstZero;

pub struct SlatsChannel {
    slats_fppu_angle_id: VariableIdentifier,
    slat_actual_position_word_id: VariableIdentifier,

    slats_demanded_angle: Angle,
    slats_feedback_angle: Angle,
}

impl SlatsChannel {
    const HANDLE_ONE_CONF_AIRSPEED_THRESHOLD_KNOTS: f64 = 205.;
    const CONF1F_TO_CONF1_AIRSPEED_THRESHOLD_KNOTS: f64 = 212.;
    const CRUISE_BAULK_AIRSPEED_THRESHOLD_KNOTS: f64 = 265.5;
    const CRUISE_BAULK_ALTITUDE_THRESHOLD_FEET: f64 = 22000.;
    const ALPHA_SPEED_LOCK_IN_AIRSPEED_THRESHOLD_KNOTS: f64 = 155.;
    const ALPHA_SPEED_LOCK_OUT_AIRSPEED_THRESHOLD_KNOTS: f64 = 161.;
    const ALPHA_SPEED_LOCK_IN_AOA_THRESHOLD_DEGREES: f64 = 9.5;
    const ALPHA_SPEED_LOCK_OUT_AOA_THRESHOLD_DEGREES: f64 = 9.2;

    const FLRS_CONFFULL_TO_CONF3_AIRSPEED_THRESHOLD_KNOTS: f64 = 184.5;
    const FLRS_CONF3_TO_CONF2S_AIRSPEED_THRESHOLD_KNOTS: f64 = 198.5;
    const FLRS_CONF2_TO_CONF1F_AIRSPEED_THRESHOLD_KNOTS: f64 = 222.5;

    pub fn new(context: &mut InitContext) -> Self {
        Self {
            slats_fppu_angle_id: context.get_identifier("SLATS_FPPU_ANGLE".to_owned()),
            slat_actual_position_word_id: context
                .get_identifier("SFCC_SLAT_ACTUAL_POSITION_WORD".to_owned()),

            slats_demanded_angle: Angle::new::<degree>(0.),
            slats_feedback_angle: Angle::new::<degree>(0.),
        }
    }

    // FIXME This is not the correct ADR input selection yet, due to missing references
    fn angle_of_attack(&self, adirs: &impl AdirsMeasurementOutputs) -> Option<Angle> {
        [1, 2, 3]
            .iter()
            .find_map(|&adiru_number| adirs.angle_of_attack(adiru_number).normal_value())
    }

    fn generate_configuration(
        &self,
        csu_monitor: &CSUMonitor,
        context: &UpdateContext,
        adirs: &impl AdirsMeasurementOutputs,
        alpha_speed_lock_active: bool,
    ) -> Angle {
        // Ignored `CSU::OutOfDetent` and `CSU::Fault` positions due to simplified SFCC.
        match (
            csu_monitor.get_previous_detent(),
            csu_monitor.get_current_detent(),
        ) {
            (CSU::Conf0 | CSU::Conf1, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    < Self::HANDLE_ONE_CONF_AIRSPEED_THRESHOLD_KNOTS
                    || context.is_on_ground() =>
            {
                Angle::new::<degree>(247.27)
            }
            (CSU::Conf0 | CSU::Conf1, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    > Self::CRUISE_BAULK_AIRSPEED_THRESHOLD_KNOTS
                    // FIXME use ADRs
                    || context.pressure_altitude().get::<foot>()
                        > Self::CRUISE_BAULK_ALTITUDE_THRESHOLD_FEET =>
            {
                Angle::ZERO
            }
            (CSU::Conf0, CSU::Conf1) => Angle::new::<degree>(247.27),
            (CSU::Conf1, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    > Self::CONF1F_TO_CONF1_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(247.27)
            }
            (CSU::Conf1, CSU::Conf1) => self.slats_demanded_angle,
            (_, CSU::Conf1)
                if context.indicated_airspeed().get::<knot>()
                    <= Self::CONF1F_TO_CONF1_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(247.27)
            }
            (_, CSU::Conf1) => Angle::new::<degree>(247.27),
            (_, CSU::Conf0) if context.is_in_flight() && alpha_speed_lock_active => {
                if context.indicated_airspeed().get::<knot>()
                    > Self::ALPHA_SPEED_LOCK_OUT_AIRSPEED_THRESHOLD_KNOTS
                    || self
                        .angle_of_attack(adirs)
                        .unwrap_or_default()
                        .get::<degree>()
                        < Self::ALPHA_SPEED_LOCK_OUT_AOA_THRESHOLD_DEGREES
                {
                    Angle::ZERO
                } else {
                    self.slats_demanded_angle
                }
            }
            (CSU::Conf1, CSU::Conf0)
            | (CSU::Conf2, CSU::Conf0)
            | (CSU::Conf3, CSU::Conf0)
            | (CSU::ConfFull, CSU::Conf0)
                if context.is_in_flight()
                    && (context.indicated_airspeed().get::<knot>()
                        < Self::ALPHA_SPEED_LOCK_IN_AIRSPEED_THRESHOLD_KNOTS
                        || self
                            .angle_of_attack(adirs)
                            .unwrap_or_default()
                            .get::<degree>()
                            > Self::ALPHA_SPEED_LOCK_IN_AOA_THRESHOLD_DEGREES) =>
            {
                Angle::new::<degree>(247.27)
            }
            (_, CSU::Conf0) => Angle::ZERO,
            (CSU::Conf1 | CSU::Conf2, CSU::Conf2)
                if context.indicated_airspeed().get::<knot>()
                    > Self::FLRS_CONF2_TO_CONF1F_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(247.27)
            }
            (CSU::Conf2, CSU::Conf2)
                if self.slats_demanded_angle == Angle::new::<degree>(247.27) =>
            {
                Angle::new::<degree>(247.27)
            }
            (CSU::Conf2 | CSU::Conf3, CSU::Conf3)
                if context.indicated_airspeed().get::<knot>()
                    > Self::FLRS_CONF3_TO_CONF2S_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(284.65)
            }
            (CSU::Conf3, CSU::Conf3)
                if self.slats_demanded_angle == Angle::new::<degree>(284.65) =>
            {
                Angle::new::<degree>(284.65)
            }
            (CSU::Conf3 | CSU::ConfFull, CSU::ConfFull)
                if context.indicated_airspeed().get::<knot>()
                    > Self::FLRS_CONFFULL_TO_CONF3_AIRSPEED_THRESHOLD_KNOTS =>
            {
                Angle::new::<degree>(284.65)
            }
            (CSU::ConfFull, CSU::ConfFull)
                if self.slats_demanded_angle == Angle::new::<degree>(284.65) =>
            {
                Angle::new::<degree>(284.65)
            }
            (from, CSU::Conf2) if from != CSU::Conf2 => Angle::new::<degree>(247.27),
            (from, CSU::Conf3) if from != CSU::Conf3 => Angle::new::<degree>(284.65),
            (from, CSU::ConfFull) if from != CSU::ConfFull => Angle::new::<degree>(284.65),
            (_, _) => self.slats_demanded_angle,
        }
    }

    pub fn update(
        &mut self,
        context: &UpdateContext,
        csu_monitor: &CSUMonitor,
        slats_feedback: &impl PositionPickoffUnit,
        adirs: &impl AdirsMeasurementOutputs,
        alpha_speed_lock_active: bool,
    ) {
        self.slats_demanded_angle =
            self.generate_configuration(csu_monitor, context, adirs, alpha_speed_lock_active);
        self.slats_feedback_angle = slats_feedback.angle();
    }

    pub fn get_demanded_angle(&self) -> Angle {
        self.slats_demanded_angle
    }

    pub fn get_feedback_angle(&self) -> Angle {
        self.slats_feedback_angle
    }

    fn slat_actual_position_word(&self) -> Arinc429Word<f64> {
        Arinc429Word::new(
            self.slats_feedback_angle.get::<degree>(),
            SignStatus::NormalOperation,
        )
    }
}
impl SimulationElement for SlatsChannel {
    fn write(&self, writer: &mut SimulatorWriter) {
        writer.write(&self.slats_fppu_angle_id, self.slats_feedback_angle);

        writer.write(
            &self.slat_actual_position_word_id,
            self.slat_actual_position_word(),
        );
    }
}
