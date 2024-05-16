// opff import
utils::import_noreturn!(common::opff::fighter_common_opff);
use super::*;
use globals::*;

unsafe fn nspecial_cancels(boma: &mut BattleObjectModuleAccessor) {
    //PM-like neutral-b canceling
    if boma.is_status(*FIGHTER_PACMAN_STATUS_KIND_SPECIAL_N_CANCEL)
    && boma.is_situation(*SITUATION_KIND_AIR)
    && WorkModule::get_int(boma, *FIGHTER_PACMAN_STATUS_SPECIAL_N_WORK_INT_NEXT_STATUS) == *FIGHTER_STATUS_KIND_ESCAPE_AIR {
        WorkModule::set_int(boma, *STATUS_KIND_NONE, *FIGHTER_PACMAN_STATUS_SPECIAL_N_WORK_INT_NEXT_STATUS);
        ControlModule::clear_command_one(boma, *FIGHTER_PAD_COMMAND_CATEGORY1, *FIGHTER_PAD_CMD_CAT1_AIR_ESCAPE);
    }
}

unsafe fn side_special_freefall(fighter: &mut L2CFighterCommon) {
    if fighter.is_prev_status(*FIGHTER_PACMAN_STATUS_KIND_SPECIAL_S_DASH)
    && fighter.is_status(*FIGHTER_PACMAN_STATUS_KIND_SPECIAL_S_RETURN)
    && !StatusModule::is_changing(fighter.module_accessor)
    && fighter.is_situation(*SITUATION_KIND_AIR)
    && CancelModule::is_enable_cancel(fighter.module_accessor)
    && WorkModule::is_flag(fighter.module_accessor, *FIGHTER_PACMAN_STATUS_SPECIAL_S_WORK_FLAG_EAT_POWER_ESA) {
        fighter.change_status_req(*FIGHTER_STATUS_KIND_FALL_SPECIAL, true);
        let cancel_module = *(fighter.module_accessor as *mut BattleObjectModuleAccessor as *mut u64).add(0x128 / 8) as *const u64;
        *(((cancel_module as u64) + 0x1c) as *mut bool) = false;  // CancelModule::is_enable_cancel = false
    }

    if fighter.is_status(*FIGHTER_PACMAN_STATUS_KIND_SPECIAL_S_RETURN)
    && !StatusModule::is_changing(fighter.module_accessor)
    && fighter.is_prev_situation(*SITUATION_KIND_AIR)
    && fighter.is_situation(*SITUATION_KIND_GROUND) {
        fighter.change_status_req(*FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL, true);
    }
}

// Allows you to land out of upB before reaching end of animation (weird vanilla behavior)
unsafe fn up_special_proper_landing(fighter: &mut L2CFighterCommon) {
    if fighter.is_status(*FIGHTER_PACMAN_STATUS_KIND_SPECIAL_HI_END)
    && fighter.is_situation(*SITUATION_KIND_GROUND) {
        fighter.change_status_req(*FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL, false);
    }
}

unsafe fn butt_bounce(fighter: &mut L2CFighterCommon) {
    if fighter.is_motion(Hash40::new("attack_air_lw"))
    && AttackModule::is_infliction_status(fighter.module_accessor, *COLLISION_KIND_MASK_HIT | *COLLISION_KIND_MASK_SHIELD)
    && fighter.motion_frame() < 40.0 {
        MotionModule::set_frame_sync_anim_cmd(fighter.module_accessor, 40.0, true, true, false);
    }
}

unsafe fn fastfall_specials(fighter: &mut L2CFighterCommon) {
    if !fighter.is_in_hitlag()
    && !StatusModule::is_changing(fighter.module_accessor)
    && fighter.is_status_one_of(&[
        *FIGHTER_STATUS_KIND_SPECIAL_N,
        *FIGHTER_STATUS_KIND_SPECIAL_LW,
        *FIGHTER_PACMAN_STATUS_KIND_SPECIAL_N_HOLD,
        *FIGHTER_PACMAN_STATUS_KIND_SPECIAL_N_CANCEL,
        *FIGHTER_PACMAN_STATUS_KIND_SPECIAL_S_REFLECT_FALL,
        *FIGHTER_PACMAN_STATUS_KIND_SPECIAL_HI_END,
        ]) 
    && fighter.is_situation(*SITUATION_KIND_AIR) {
        fighter.sub_air_check_dive();
        if fighter.is_flag(*FIGHTER_STATUS_WORK_ID_FLAG_RESERVE_DIVE) {
            if [*FIGHTER_KINETIC_TYPE_MOTION_AIR, *FIGHTER_KINETIC_TYPE_MOTION_AIR_ANGLE].contains(&KineticModule::get_kinetic_type(fighter.module_accessor)) {
                fighter.clear_lua_stack();
                lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_MOTION);
                let speed_y = app::sv_kinetic_energy::get_speed_y(fighter.lua_state_agent);

                fighter.clear_lua_stack();
                lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_GRAVITY, ENERGY_GRAVITY_RESET_TYPE_GRAVITY, 0.0, speed_y, 0.0, 0.0, 0.0);
                app::sv_kinetic_energy::reset_energy(fighter.lua_state_agent);
                
                fighter.clear_lua_stack();
                lua_args!(fighter, FIGHTER_KINETIC_ENERGY_ID_GRAVITY);
                app::sv_kinetic_energy::enable(fighter.lua_state_agent);

                KineticUtility::clear_unable_energy(*FIGHTER_KINETIC_ENERGY_ID_MOTION, fighter.module_accessor);
            }
        }
    }
}

pub unsafe fn moveset(fighter: &mut smash::lua2cpp::L2CFighterCommon, boma: &mut BattleObjectModuleAccessor, id: usize, cat: [i32 ; 4], status_kind: i32, situation_kind: i32, motion_kind: u64, stick_x: f32, stick_y: f32, facing: f32, frame: f32) {
    nspecial_cancels(boma);
    side_special_freefall(fighter);
    up_special_proper_landing(fighter);
    butt_bounce(fighter);
    fastfall_specials(fighter);
}

pub extern "C" fn pacman_frame_wrapper(fighter: &mut smash::lua2cpp::L2CFighterCommon) {
    unsafe {
        common::opff::fighter_common_opff(fighter);
		pacman_frame(fighter)
    }
}

pub unsafe fn pacman_frame(fighter: &mut smash::lua2cpp::L2CFighterCommon) {
    if let Some(info) = FrameInfo::update_and_get(fighter) {
        moveset(fighter, &mut *info.boma, info.id, info.cat, info.status_kind, info.situation_kind, info.motion_kind.hash, info.stick_x, info.stick_y, info.facing, info.frame);
    }
}

pub fn install(agent: &mut Agent) {
    agent.on_line(Main, pacman_frame_wrapper);
}