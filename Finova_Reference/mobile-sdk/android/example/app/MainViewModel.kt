package com.finova.example.viewmodel

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.finova.example.MainUiState
import com.finova.example.MiningData
import com.finova.example.MiningHistory
import com.finova.example.XPData
import com.finova.example.XPActivity
import com.finova.example.ReferralData
import com.finova.example.Referral
import com.finova.example.NFTData
import com.finova.example.NFT
import com.finova.example.UserData
import com.finova.example.WalletData
import com.finova.example.Achievement
import com.finova.sdk.FinovaSDK
import com.finova.sdk.models.*
import com.finova.sdk.services.*
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    @ApplicationContext private val context: Context,
    private val finovaSDK: FinovaSDK
) : ViewModel() {

    private val _uiState = MutableStateFlow(MainUiState())
    val uiState: StateFlow<MainUiState> = _uiState.asStateFlow()

    private val miningService = finovaSDK.miningService
    private val xpService = finovaSDK.xpService
    private val referralService = finovaSDK.referralService
    private val nftService = finovaSDK.nftService
    private val userService = finovaSDK.userService
    private val walletService = finovaSDK.walletService

    init {
        loadInitialData()
        startPeriodicUpdates()
    }

    private fun loadInitialData() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true)
            
            try {
                // Load all data concurrently
                val miningData = loadMiningData()
                val xpData = loadXPData()
                val referralData = loadReferralData()
                val nftData = loadNFTData()
                val userData = loadUserData()
                val walletData = loadWalletData()
                
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    miningData = miningData,
                    xpData = xpData,
                    referralData = referralData,
                    nftData = nftData,
                    userData = userData,
                    walletData = walletData
                )
                
                // Load additional data
                loadMiningHistory()
                loadXPActivities()
                loadReferrals()
                loadNFTs()
                loadAchievements()
                
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    isLoading = false,
                    error = "Failed to load data: ${e.message}"
                )
            }
        }
    }

    private fun startPeriodicUpdates() {
        viewModelScope.launch {
            while (true) {
                delay(30000) // Update every 30 seconds
                try {
                    updateMiningData()
                    updateWalletData()
                } catch (e: Exception) {
                    // Silent update failure
                }
            }
        }
    }

    // Mining Functions
    fun startMining() {
        viewModelScope.launch {
            try {
                val result = miningService.startMining()
                if (result.isSuccess) {
                    updateMiningData()
                } else {
                    _uiState.value = _uiState.value.copy(
                        error = "Failed to start mining: ${result.error}"
                    )
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = "Mining error: ${e.message}"
                )
            }
        }
    }

    fun stopMining() {
        viewModelScope.launch {
            try {
                val result = miningService.stopMining()
                if (result.isSuccess) {
                    updateMiningData()
                } else {
                    _uiState.value = _uiState.value.copy(
                        error = "Failed to stop mining: ${result.error}"
                    )
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = "Mining error: ${e.message}"
                )
            }
        }
    }

    fun claimRewards() {
        viewModelScope.launch {
            try {
                val result = miningService.claimRewards()
                if (result.isSuccess) {
                    updateMiningData()
                    updateWalletData()
                    _uiState.value = _uiState.value.copy(
                        error = "Rewards claimed successfully!"
                    )
                } else {
                    _uiState.value = _uiState.value.copy(
                        error = "Failed to claim rewards: ${result.error}"
                    )
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = "Claim error: ${e.message}"
                )
            }
        }
    }

    fun useSpecialCard() {
        viewModelScope.launch {
            try {
                // Get available special cards
                val cards = nftService.getSpecialCards()
                if (cards.isNotEmpty()) {
                    val result = nftService.useSpecialCard(cards.first().id)
                    if (result.isSuccess) {
                        updateMiningData()
                        updateNFTData()
                        _uiState.value = _uiState.value.copy(
                            error = "Special card activated!"
                        )
                    }
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = "Card usage error: ${e.message}"
                )
            }
        }
    }

    // XP Functions
    fun completeXPActivity(activityId: String) {
        viewModelScope.launch {
            try {
                val result = xpService.completeActivity(activityId)
                if (result.isSuccess) {
                    updateXPData()
                    loadXPActivities()
                    _uiState.value = _uiState.value.copy(
                        error = "Activity completed! +${result.data?.xpGained} XP"
                    )
                } else {
                    _uiState.value = _uiState.value.copy(
                        error = "Failed to complete activity: ${result.error}"
                    )
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = "XP error: ${e.message}"
                )
            }
        }
    }

    fun refreshXPActivities() {
        viewModelScope.launch {
            loadXPActivities()
        }
    }

    // Referral Functions
    fun copyReferralCode(code: String) {
        try {
            val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            val clip = ClipData.newPlainText("Referral Code", code)
            clipboard.setPrimaryClip(clip)
            _uiState.value = _uiState.value.copy(
                error = "Referral code copied to clipboard!"
            )
        } catch (e: Exception) {
            _uiState.value = _uiState.value.copy(
                error = "Failed to copy referral code"
            )
        }
    }

    fun shareReferralCode(code: String) {
        try {
            val shareIntent = Intent().apply {
                action = Intent.ACTION_SEND
                type = "text/plain"
                putExtra(Intent.EXTRA_TEXT, "Join Finova Network with my code: $code\nhttps://finova.network/ref/$code")
                putExtra(Intent.EXTRA_SUBJECT, "Join Finova Network!")
            }
            val chooserIntent = Intent.createChooser(shareIntent, "Share Referral Code")
            chooserIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK
            context.startActivity(chooserIntent)
        } catch (e: Exception) {
            _uiState.value = _uiState.value.copy(
                error = "Failed to share referral code"
            )
        }
    }

    // NFT Functions
    fun useNFT(nftId: String) {
        viewModelScope.launch {
            try {
                val result = nftService.useNFT(nftId)
                if (result.isSuccess) {
                    updateNFTData()
                    updateMiningData()
                    _uiState.value = _uiState.value.copy(
                        error = "NFT used successfully!"
                    )
                } else {
                    _uiState.value = _uiState.value.copy(
                        error = "Failed to use NFT: ${result.error}"
                    )
                }
            } catch (e: Exception) {
                _uiState.value = _uiState.value.copy(
                    error = "NFT error: ${e.message}"
                )
            }
        }
    }

    fun openMarketplace() {
        // Navigate to NFT marketplace
        _uiState.value = _uiState.value.copy(
            error = "Opening NFT Marketplace..."
        )
    }

    fun openCardPacks() {
        // Navigate to card pack purchase
        _uiState.value = _uiState.value.copy(
            error = "Opening Card Packs..."
        )
    }

    // Wallet Functions
    fun refreshWallet() {
        viewModelScope.launch {
            updateWalletData()
        }
    }

    fun openDeposit() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Deposit..."
        )
    }

    fun openWithdraw() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Withdraw..."
        )
    }

    fun openSwap() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Token Swap..."
        )
    }

    // Profile Functions
    fun startKYCVerification() {
        _uiState.value = _uiState.value.copy(
            error = "Starting KYC Verification..."
        )
    }

    // Settings Functions
    fun openStaking() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Staking..."
        )
    }

    fun openSettings() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Settings..."
        )
    }

    fun openNotificationSettings() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Notification Settings..."
        )
    }

    fun openSecuritySettings() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Security Settings..."
        )
    }

    fun openSocialSettings() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Social Account Settings..."
        )
    }

    fun openSupport() {
        _uiState.value = _uiState.value.copy(
            error = "Opening Help & Support..."
        )
    }

    fun openAbout() {
        _uiState.value = _uiState.value.copy(
            error = "Opening About..."
        )
    }

    fun viewAllAchievements() {
        _uiState.value = _uiState.value.copy(
            error = "Opening All Achievements..."
        )
    }

    // Utility Functions
    fun refreshData() {
        loadInitialData()
    }

    fun clearError() {
        _uiState.value = _uiState.value.copy(error = null)
    }

    // Private Data Loading Functions
    private suspend fun loadMiningData(): MiningData {
        val miningStatus = miningService.getMiningStatus()
        return MiningData(
            isActive = miningStatus.isActive,
            currentRate = miningStatus.currentRate,
            totalMined = miningStatus.totalMined,
            todayProgress = miningStatus.todayProgress.toFloat(),
            phase = miningStatus.phase,
            multipliers = miningStatus.multipliers
        )
    }

    private suspend fun loadXPData(): XPData {
        val xpStatus = xpService.getXPStatus()
        return XPData(
            currentLevel = xpStatus.currentLevel,
            totalXP = xpStatus.totalXP,
            badgeTier = xpStatus.badgeTier,
            levelProgress = xpStatus.levelProgress.toFloat(),
            xpToNextLevel = xpStatus.xpToNextLevel,
            miningMultiplier = xpStatus.miningMultiplier,
            dailyFinCap = xpStatus.dailyFinCap,
            specialUnlocks = xpStatus.specialUnlocks
        )
    }

    private suspend fun loadReferralData(): ReferralData {
        val referralStatus = referralService.getReferralStatus()
        return ReferralData(
            totalRP = referralStatus.totalRP,
            rpTier = referralStatus.rpTier,
            totalReferrals = referralStatus.totalReferrals,
            activeReferrals = referralStatus.activeReferrals,
            networkBonus = referralStatus.networkBonus,
            qualityScore = referralStatus.qualityScore,
            referralCode = referralStatus.referralCode,
            referralLink = referralStatus.referralLink
        )
    }

    private suspend fun loadNFTData(): NFTData {
        val nftStatus = nftService.getNFTStatus()
        return NFTData(
            totalNFTs = nftStatus.totalNFTs,
            specialCards = nftStatus.specialCards,
            totalValue = nftStatus.totalValue
        )
    }

    private suspend fun loadUserData(): UserData {
        val userProfile = userService.getUserProfile()
        return UserData(
            username = userProfile.username,
            userId = userProfile.userId,
            joinedDate = userProfile.joinedDate,
            kycStatus = userProfile.kycStatus,
            isKYCVerified = userProfile.isKYCVerified,
            humanScore = userProfile.humanScore,
            globalRank = userProfile.globalRank,
            totalValue = userProfile.totalValue
        )
    }

    private suspend fun loadWalletData(): WalletData {
        val walletBalance = walletService.getWalletBalance()
        return WalletData(
            finBalance = walletBalance.finBalance,
            sfinBalance = walletBalance.sfinBalance,
            usdfinBalance = walletBalance.usdfinBalance,
            solBalance = walletBalance.solBalance
        )
    }

    private fun loadMiningHistory() {
        viewModelScope.launch {
            try {
                val history = miningService.getMiningHistory()
                _uiState.value = _uiState.value.copy(
                    miningHistory = history.map { h ->
                        MiningHistory(
                            timestamp = h.timestamp,
                            amount = h.amount,
                            type = h.type
                        )
                    }
                )
            } catch (e: Exception) {
                // Silent failure for history
            }
        }
    }

    private fun loadXPActivities() {
        viewModelScope.launch {
            try {
                val activities = xpService.getDailyActivities()
                _uiState.value = _uiState.value.copy(
                    xpActivities = activities.map { a ->
                        XPActivity(
                            id = a.id,
                            name = a.name,
                            baseXP = a.baseXP,
                            dailyLimit = a.dailyLimit,
                            completed = a.completed,
                            platform = a.platform
                        )
                    }
                )
            } catch (e: Exception) {
                // Silent failure for activities
            }
        }
    }

    private fun loadReferrals() {
        viewModelScope.launch {
            try {
                val referrals = referralService.getReferralNetwork()
                _uiState.value = _uiState.value.copy(
                    referrals = referrals.map { r ->
                        Referral(
                            username = r.username,
                            level = r.level,
                            joinedDate = r.joinedDate,
                            isActive = r.isActive,
                            rpEarned = r.rpEarned
                        )
                    }
                )
            } catch (e: Exception) {
                // Silent failure for referrals
            }
        }
    }

    private fun loadNFTs() {
        viewModelScope.launch {
            try {
                val nfts = nftService.getUserNFTs()
                _uiState.value = _uiState.value.copy(
                    nfts = nfts.map { n ->
                        NFT(
                            id = n.id,
                            name = n.name,
                            rarity = n.rarity,
                            category = n.category,
                            effect = n.effect,
                            value = n.value,
                            isUsable = n.isUsable
                        )
                    }
                )
            } catch (e: Exception) {
                // Silent failure for NFTs
            }
        }
    }

    private fun loadAchievements() {
        viewModelScope.launch {
            try {
                val achievements = userService.getUserAchievements()
                _uiState.value = _uiState.value.copy(
                    achievements = achievements.map { a ->
                        Achievement(
                            name = a.name,
                            description = a.description,
                            icon = a.icon,
                            rarity = a.rarity,
                            isCompleted = a.isCompleted,
                            reward = a.reward
                        )
                    }
                )
            } catch (e: Exception) {
                // Silent failure for achievements
            }
        }
    }

    private fun updateMiningData() {
        viewModelScope.launch {
            try {
                val miningData = loadMiningData()
                _uiState.value = _uiState.value.copy(miningData = miningData)
            } catch (e: Exception) {
                // Silent update failure
            }
        }
    }

    private fun updateXPData() {
        viewModelScope.launch {
            try {
                val xpData = loadXPData()
                _uiState.value = _uiState.value.copy(xpData = xpData)
            } catch (e: Exception) {
                // Silent update failure
            }
        }
    }

    private fun updateNFTData() {
        viewModelScope.launch {
            try {
                val nftData = loadNFTData()
                _uiState.value = _uiState.value.copy(nftData = nftData)
                loadNFTs() // Refresh NFT list
            } catch (e: Exception) {
                // Silent update failure
            }
        }
    }

    private fun updateWalletData() {
        viewModelScope.launch {
            try {
                val walletData = loadWalletData()
                _uiState.value = _uiState.value.copy(walletData = walletData)
            } catch (e: Exception) {
                // Silent update failure
            }
        }
    }
}
