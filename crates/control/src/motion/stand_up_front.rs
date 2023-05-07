use color_eyre::Result;
use context_attribute::context;
use framework::MainOutput;
use motionfile::{MotionFile, MotionInterpolator};
use types::{ConditionInput, JointsVelocity};
use types::{
    CycleTime, Joints, MotionCommand, MotionFinished, MotionSelection, MotionType, SensorData,
};

pub struct StandUpFront {
    interpolator: MotionInterpolator<Joints<f32>>,
}

#[context]
pub struct CreationContext {}

#[context]
pub struct CycleContext {
    pub condition_input: Input<ConditionInput, "condition_input">,
    pub cycle_time: Input<CycleTime, "cycle_time">,
    pub motion_command: Input<MotionCommand, "motion_command">,
    pub motion_selection: Input<MotionSelection, "motion_selection">,
    pub sensor_data: Input<SensorData, "sensor_data">,

    pub gyro_low_pass_filter_coefficient:
        Parameter<f32, "stand_up.gyro_low_pass_filter_coefficient">,
    pub gyro_low_pass_filter_tolerance: Parameter<f32, "stand_up.gyro_low_pass_filter_tolerance">,
    pub maximum_velocity: Parameter<JointsVelocity, "maximum_joint_velocities">,

    pub motion_finished: PersistentState<MotionFinished, "motion_finished">,
    pub should_exit_stand_up_front: PersistentState<bool, "should_exit_stand_up_front">,
}

#[context]
#[derive(Default)]
pub struct MainOutputs {
    pub stand_up_front_positions: MainOutput<Joints<f32>>,
}

impl StandUpFront {
    pub fn new(_context: CreationContext) -> Result<Self> {
        Ok(Self {
            interpolator: MotionFile::from_path("etc/motions/stand_up_front.json")?.try_into()?,
        })
    }

    pub fn compute_stand_up_front(&mut self, context: CycleContext) -> Result<Joints<f32>> {
        let last_cycle_duration = context.cycle_time.last_cycle_duration;
        let condition_input = context.condition_input;

        context.motion_finished[MotionType::StandUpFront] = false;

        self.interpolator
            .advance_by(last_cycle_duration, condition_input);

        if self.interpolator.is_finished() {
            context.motion_finished[MotionType::StandUpFront] = true;
        }

        Ok(self.interpolator.value())
    }

    pub fn cycle(&mut self, context: CycleContext) -> Result<MainOutputs> {
        let current_position =
            if let MotionType::StandUpFront = context.motion_selection.current_motion {
                self.compute_stand_up_front(context)?
            } else {
                self.interpolator.reset();
                self.interpolator.value()
            };
        Ok(MainOutputs {
            stand_up_front_positions: current_position.into(),
        })
    }
}
