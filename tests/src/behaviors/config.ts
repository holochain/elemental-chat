const path = require('path')
const { Config } = require('@holochain/tryorama')
import { configBatchSimple } from '@holochain/tryorama-stress-utils'

const dnaPath = path.join(__dirname, '../../../elemental-chat.dna.gz')
console.log(`PAHT: ${dnaPath}`)
const dnaUri = 'https://github.com/holochain/elemental-chat/fixme.gz'

const commonConfig = {
    logger: {
        type: 'info'
    }
}

export const batcher = (isRemote, numConductors, instancesPerConductor) => configBatchSimple(
    numConductors,
    instancesPerConductor,
    Config.dna(isRemote ? dnaUri : dnaPath, 'app'),
    commonConfig
)
