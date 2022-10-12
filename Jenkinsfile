#!groovy

pipeline {

  agent {
        label 'benchmark'
      }
  options { disableConcurrentBuilds() }
   triggers {
        issueCommentTrigger('^\\/run-benchmark.*')
    }
  stages {
    // node configuration
    stage('Run Benchmarks') {
      steps {
        deleteDir()
        checkout scm
        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install stable && rustup default stable && rustup target add wasm32-unknown-unknown --toolchain stable && make benchmarks'
        sh "git config user.email \"jenkins@frequency.xyz\""
        sh "git config user.name \"Jenkins\""
         sshagent(credentials: ['jenkins-2022-03-01']) {
             sh ' git fetch && git checkout -b $CHANGE_BRANCH && git add . && git commit -am"Updating Benchmark Files" && git push origin HEAD'
            }

      }
    }

  }
}
