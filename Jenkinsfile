#!groovy

pipeline {

  agent {
        label 'benchmark'
      }
   triggers {
        issueCommentTrigger('^release-benchmark*')
    }
  stages {
    // node configuration
    stage('node rust config') {
      steps {
        deleteDir()
        checkout scm
        result = sh (script: "git log -1 | grep '\\[runtime-benchmarks\\]'", returnStatus: true) 
        if (result != 0) {
    echo "not performing build..."
      }
       else {
        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown --toolchain nightly && make benchmarks'
        sh "git config user.email \"jenkins@frequency.xyz\""
        sh "git config user.name \"Jenkins\""
         sshagent(credentials: ['jenkins-2022-03-01']) {
             sh ' git fetch && git checkout -b $CHANGE_BRANCH && git add . && git commit -am"Updating Benchmark Files" && git push origin HEAD'
            }

      }
    }

  }
}
}
