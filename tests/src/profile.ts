import { InstalledHapp } from '@holochain/tryorama'
import path = require('path')
import { InstallAppRequest } from '@holochain/conductor-api'
import * as _ from 'lodash'
const delay = ms => new Promise(r => setTimeout(r, ms))
import { RETRY_DELAY, RETRY_COUNT, localConductorConfig, networkedConductorConfig, installation1agent, installation2agent, installAgents } from './common'

module.exports = (orchestrator) => {

  orchestrator.registerScenario('test profile zomes', async (s, t) => {
    // spawn the conductor process
    const [ conductor ] = await s.players([localConductorConfig])
    let [alice_chat_happ, bobbo_chat_happ] = await installAgents(conductor,  ["alice", 'bobbo'])

    const [alice] = alice_chat_happ.cells
    const [bobbo] = bobbo_chat_happ.cells

    // Create a channel
    const profile_input = {
      nickname: "Alice",
      avatar_url: "https://alice.img"
    };
    let profile_hash;

    try{
      profile_hash = await alice.call('profile', 'update_my_profile', profile_input);
      console.log("PROFILE_Hash:", profile_hash);
      t.ok(profile_hash)
    } catch(e) {
      console.error("Error: ", e);
      t.fail()
    }

    let a_check_a_profile = await alice.call('profile', 'get_my_profile', null);
    console.log("Alice checks her profile:", a_check_a_profile);
    t.ok(a_check_a_profile)
    t.equal(profile_input.nickname, a_check_a_profile.nickname)
    t.equal(profile_input.avatar_url, a_check_a_profile.avatar_url)

    let bobbo_check_alice_profile = await bobbo.call('profile', 'get_profile', a_check_a_profile.agent_address);
    console.log("Bobbo checks alice's profile:", bobbo_check_alice_profile);
    t.ok(bobbo_check_alice_profile)
    t.equal(profile_input.nickname, bobbo_check_alice_profile.nickname)
    t.equal(profile_input.avatar_url, bobbo_check_alice_profile.avatar_url)

    await delay(1000)
    const updated_profile_input_1 = {
      nickname: "Alicia",
      avatar_url: "https://alicia.img"
    };
    profile_hash = await alice.call('profile', 'update_my_profile', updated_profile_input_1);
    console.log("PROFILE_Hash:", profile_hash);
    t.ok(profile_hash)

    await delay(1000)
    a_check_a_profile = await alice.call('profile', 'get_my_profile', null);
    console.log("Alice checks her updated profile:", a_check_a_profile);
    t.ok(a_check_a_profile)
    t.equal(updated_profile_input_1.nickname, a_check_a_profile.nickname)
    t.equal(updated_profile_input_1.avatar_url, a_check_a_profile.avatar_url)

    bobbo_check_alice_profile = await bobbo.call('profile', 'get_profile', a_check_a_profile.agent_address);
    console.log("Bobbo checks alice's updated profile:", bobbo_check_alice_profile);
    t.ok(bobbo_check_alice_profile)
    t.equal(updated_profile_input_1.nickname, bobbo_check_alice_profile.nickname)
    t.equal(updated_profile_input_1.avatar_url, bobbo_check_alice_profile.avatar_url)

    await delay(1000)
    const updated_profile_input_2 = {
      nickname: "Alexandria",
      avatar_url: "https://alexandria.img"
    };
    profile_hash = await alice.call('profile', 'update_my_profile', updated_profile_input_2);
    console.log("PROFILE_Hash:", profile_hash);
    t.ok(profile_hash)

    await delay(1000)
    a_check_a_profile = await alice.call('profile', 'get_my_profile', null);
    console.log("Alice checks her updated profile:", a_check_a_profile);
    t.ok(a_check_a_profile)
    t.equal(updated_profile_input_2.nickname, a_check_a_profile.nickname)
    t.equal(updated_profile_input_2.avatar_url, a_check_a_profile.avatar_url)

    bobbo_check_alice_profile = await bobbo.call('profile', 'get_profile', a_check_a_profile.agent_address);
    console.log("Bobbo checks alice's updated profile:", bobbo_check_alice_profile);
    t.ok(bobbo_check_alice_profile)
    t.equal(updated_profile_input_2.nickname, bobbo_check_alice_profile.nickname)
    t.equal(updated_profile_input_2.avatar_url, bobbo_check_alice_profile.avatar_url)
  })
}
