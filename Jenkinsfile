#!groovy

pipeline {

  agent {
        label 'benchmark'
      }
   triggers {
        issueCommentTrigger('^\\/run-benchmark.*')
    }
  stages {
    // node configuration
    stage('node rust config') {
      steps {
        deleteDir()
        checkout scm
        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown --toolchain nightly && make benchmarks'
        sh "git config user.email \"jenkins@frequency.xyz\""
        sh "git config user.name \"Jenkins\""
         sshagent(credentials: ['jenkins-2022-03-01']) {
             sh ' git fetch && git checkout -b $CHANGE_BRANCH && git add . && git commit -am"Updating Benchmark Files" && git push origin HEAD'
            }

      }
    }

  }
	    // Post-build actions
    post {
        always {
            script {
                BUILD_USER = getBuildUser()
            }
            echo 'I will always say hello in the console.'
            slackSend channel: '#slack-test-channel',
                color: COLOR_MAP[currentBuild.currentResult],
                message: "*${currentBuild.currentResult}:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${BUILD_USER}\n More info at: ${env.BUILD_URL}"
        }
    }
}
