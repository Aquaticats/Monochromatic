#!/bin/sh
echo "start" >> '/work/scratch/fixture/hook-lock-dead/hook.log'
if [ -n "$BLOCK_HOOK" ]; then sleep 600; fi
echo "end" >> '/work/scratch/fixture/hook-lock-dead/hook.log'
