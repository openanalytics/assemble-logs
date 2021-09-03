pipeline {

    agent {
        kubernetes {
            yamlFile 'kubernetesPod.yaml'
        }
    }

    options {
        authorizationMatrix(['hudson.model.Item.Build:hci', 'hudson.model.Item.Read:hci'])
        buildDiscarder(logRotator(numToKeepStr: '3'))
    }

    environment {
        RUST_BACKTRACE = 1
        GIT_TERMINAL_PROMPT = 0
        JQ_LIB_DIR = "/usr/lib/x86_64-linux-gnu/libjq.so"
        NAME = "assemble-logs"
    }
    
    stages {
        
        stage('init') {
            steps {
                withCredentials([usernameColonPassword(credentialsId: 'oa-jenkins', variable: 'USERPASS')]) {
                    container('musl') {
                        timestamps {
                            // Download and extract cache
                            sh "aws s3 cp s3://oa-infrastructure-cache/${NAME}/${env.BRANCH_NAME}/cache.tar ${env.WORKSPACE}/cache.tar --region eu-west-1 --quiet || echo 'Failed downloading cache'"
                            sh "tar -Pxf cache.tar || echo 'Failed extracting cache'"

                            // Install libjq-dev
                            sh "sudo apt-get update"
                            sh "sudo apt-get --yes --force-yes install libjq-dev"
                            sh "sudo apt-get --yes --force-yes install libonig-dev"
                        }
                    }
                }
            }
        }
        stage('build'){
        
            steps {
                container('musl') {
                    timestamps {
                        sh "cargo build --release"
                    }
                }
            }
        }
        stage('deploy to nexus') {
            steps {
                withCredentials([usernameColonPassword(credentialsId: 'oa-jenkins', variable: 'USERPASS')]) {
                    sh "curl -v -u $USERPASS --upload-file target/x86_64-unknown-linux-musl/release/${NAME} https://nexus.openanalytics.eu/repository/minos/${NAME}/${env.BRANCH_NAME}/${env.BUILD_NUMBER}/${NAME}"
                    sh "curl -v -u $USERPASS --upload-file target/x86_64-unknown-linux-musl/release/${NAME} https://nexus.openanalytics.eu/repository/minos/${NAME}/${env.BRANCH_NAME}/${NAME}-latest"
                }
            }
        }
    }

    post {
        success {
            archiveArtifacts artifacts: "target/x86_64-unknown-linux-musl/release/${NAME}", fingerprint: true
        }
        always {
            container('musl') {
                timestamps {
                    sh "tar -Pcf cache.tar ${env.WORKSPACE}/target ~/.cargo/git ~/.cargo/registry"
                    sh "aws s3 cp ${env.WORKSPACE}/cache.tar s3://oa-infrastructure-cache/${NAME}/${env.BRANCH_NAME}/cache.tar --region eu-west-1 --quiet"
                    sh 'du -sch cache.tar'
                }
            }
        }
    }
}
