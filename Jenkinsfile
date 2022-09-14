#!groovy

pipeline {

  agent
      {
     label 'benchmark'
     }
   triggers {
        issueCommentTrigger('^\\/run-benchmark.*')
    }
    parameters {

    string(name: 'SLACK_CHANNEL_1',
           description: 'Default Slack channel to send messages to',
           defaultValue: '#jenkins-job-notification')

  } 
  environment {

    // Slack configuration
    SLACK_COLOR_DANGER  = '#E01563'
    SLACK_COLOR_INFO    = '#6ECADC'
    SLACK_COLOR_WARNING = '#FFC300'
    SLACK_COLOR_GOOD    = '#3EB991'
    // String comparision
    GIT_CMP = '/run-benchmark'
  } 

  stages {
    // checking git comment
    stage ('checking git comment') {
         steps {  
             script {
                 deleteDir()
             checkout scm
             env.GIT_COMMENT = sh(script:'git log -1 HEAD --pretty=format:%s', returnStdout: true).trim()

}
}
}
    stage('node rust config') {
      when { expression { return env.GIT_COMMENT == env.GIT_CMP } }
      steps {
        deleteDir()
        checkout scm
               // get user that has started the build
           wrap([$class: 'BuildUser']) { script { env.USER_ID = "${BUILD_USER_ID}" } }
                 // first of all, notify the team
           slackSend (color: "${env.SLACK_COLOR_INFO}",
                   channel: "${params.SLACK_CHANNEL_1}",
                   message: "*STARTED:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.USER_ID}\n More info at: (<${env.BUILD_URL}|Open>)")

        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -snf /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && rustup target add wasm32-unknown-unknown --toolchain nightly && make benchmarks'
        sh "git config user.email \"jenkins@frequency.xyz\""
        sh "git config user.name \"Jenkins\""
         sshagent(credentials: ['jenkins-2022-03-01']) {
             sh ' git fetch && git checkout -b $CHANGE_BRANCH && git add . && git commit -am"Updating Benchmark Files" && git push origin HEAD'
            }

      }
    }

  }
post {

    aborted {

      echo "Sending message to Slack"
      slackSend (color: "${env.SLACK_COLOR_WARNING}",
                 channel: "${params.SLACK_CHANNEL_1}",
                 message: "*ABORTED:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.USER_ID}\n More info at: ${env.BUILD_URL}")
    } // aborted

    failure {

      echo "Sending message to Slack"
      slackSend (color: "${env.SLACK_COLOR_DANGER}",
                 channel: "${params.SLACK_CHANNEL_1}",
                 message: "*FAILED:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.USER_ID}\n More info at: ${env.BUILD_URL}")
    } // failure

    success {
      echo "Sending message to Slack"
      slackSend (color: "${env.SLACK_COLOR_GOOD}",
                 channel: "${params.SLACK_CHANNEL_1}",
                 message: "*SUCCESS:* Job ${env.JOB_NAME} build ${env.BUILD_NUMBER} by ${env.USER_ID}\n More info at: ${env.BUILD_URL}")
    } // success

  } // post
}

